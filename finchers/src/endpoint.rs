//! Components for constructing `Endpoint`.

pub mod syntax;

mod and;
mod and_then;
mod boxed;
mod err_into;
mod map;
mod map_err;
mod or;
mod or_else;
mod or_strict;

// re-exports
pub use self::{
    and::And,
    and_then::AndThen,
    boxed::{EndpointObj, LocalEndpointObj},
    err_into::ErrInto,
    map::Map,
    map_err::MapErr,
    or::Or,
    or_else::OrElse,
    or_strict::OrStrict,
};

// ====

use {
    self::syntax::EncodedStr,
    crate::{common::Tuple, error::Error},
    futures::{Future, IntoFuture, Poll},
    http::Request,
    std::{marker::PhantomData, rc::Rc, sync::Arc},
};

/// A marker trait indicating that the implementor has an implementation of `Endpoint<Bd>`.
pub trait IsEndpoint {}

impl<'a, E: IsEndpoint + ?Sized> IsEndpoint for &'a E {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Box<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Rc<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Arc<E> {}

/// Trait representing an endpoint.
pub trait Endpoint<Bd>: IsEndpoint {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The error type associated with this endpoint.
    type Error: Into<Error>;

    /// The type of `Action` which will be returned from `Self::apply`.
    type Action: EndpointAction<
        Bd, //
        Output = Self::Output,
        Error = Self::Error,
    >;

    /// Spawns an instance of `Action` applied to an incoming request.
    fn action(&self) -> Self::Action;

    /// Add an annotation that the associated type `Output` is fixed to `T`.
    #[inline(always)]
    fn with_output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Bd, Output = T> + Sized,
    {
        self
    }
}

impl<'a, E, Bd> Endpoint<Bd> for &'a E
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Box<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Rc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Arc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

/// Create an endpoint from a function which takes the reference to `ApplyContext`
/// and returns a future.
///
/// The endpoint created by this function will wrap the result of future into a tuple.
/// If you want to return the result without wrapping, use `apply_raw` instead.
pub fn apply_fn<Bd, R>(
    f: impl Fn() -> R,
) -> impl Endpoint<
    Bd, //
    Output = R::Output,
    Error = R::Error,
    Action = R,
>
where
    R: EndpointAction<Bd>,
{
    #[allow(missing_debug_implementations)]
    struct ApplyEndpoint<F>(F);

    impl<F> IsEndpoint for ApplyEndpoint<F> {}

    impl<F, Bd, R> Endpoint<Bd> for ApplyEndpoint<F>
    where
        F: Fn() -> R,
        R: EndpointAction<Bd>,
    {
        type Output = R::Output;
        type Error = R::Error;
        type Action = R;

        fn action(&self) -> Self::Action {
            (self.0)()
        }
    }

    ApplyEndpoint(f)
}

/// Create an endpoint which simply returns an unit (`()`).
#[inline]
pub fn unit<Bd>() -> impl Endpoint<
    Bd,
    Output = (),
    Error = crate::util::Never,
    Action = Oneshot<self::unit::UnitAction>, // private
> {
    apply_fn(|| self::unit::UnitAction(()).into_action())
}

mod unit {
    use super::*;

    #[allow(missing_debug_implementations)]
    pub struct UnitAction(pub(super) ());

    impl OneshotAction for UnitAction {
        type Output = ();
        type Error = crate::util::Never;

        fn apply(self, _: &mut ApplyContext<'_>) -> Result<Self::Output, Self::Error> {
            Ok(())
        }
    }
}

/// Create an endpoint which simply clones the specified value.
///
/// # Examples
///
/// ```ignore
/// # #[macro_use]
/// # extern crate finchers;
/// # extern crate futures;
/// # use finchers::prelude::*;
/// # use finchers::endpoint::value;
/// #
/// #[derive(Clone)]
/// struct Conn {
///     // ...
/// #   _p: (),
/// }
///
/// # fn main() {
/// let conn = {
///     // do some stuff...
/// #   Conn { _p: () }
/// };
///
/// let endpoint = path!(@get / "posts" / u32 /)
///     .and(value(conn))
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       drop(id);
/// #       futures::future::ok::<_, finchers::error::Never>(conn)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn value<Bd, T: Clone>(
    x: T,
) -> impl Endpoint<
    Bd,
    Output = (T,),
    Error = crate::util::Never,
    Action = Oneshot<self::value::ValueAction<T>>, // private
> {
    apply_fn(move || self::value::ValueAction { x: x.clone() }.into_action())
}

mod value {
    use super::*;

    // not a public API.
    #[derive(Debug)]
    pub struct ValueAction<T> {
        pub(super) x: T,
    }

    impl<T> OneshotAction for ValueAction<T> {
        type Output = (T,);
        type Error = crate::util::Never;

        fn apply(self, _: &mut ApplyContext<'_>) -> Result<Self::Output, Self::Error> {
            Ok((self.x,))
        }
    }
}

/// Create an endpoint from the specified function which returns a `Future`.
pub fn lazy<Bd, R>(
    f: impl Fn() -> R,
) -> impl Endpoint<
    Bd,
    Output = (R::Item,),
    Error = R::Error,
    Action = self::lazy::LazyAction<R::Future>, // private
>
where
    R: IntoFuture,
    R::Error: Into<Error>,
{
    apply_fn(move || self::lazy::LazyAction {
        future: f().into_future(),
    })
}

mod lazy {
    use super::*;

    #[derive(Debug)]
    pub struct LazyAction<F> {
        pub(super) future: F,
    }

    impl<F, Bd> EndpointAction<Bd> for LazyAction<F>
    where
        F: Future,
        F::Error: Into<Error>,
    {
        type Output = (F::Item,);
        type Error = F::Error;

        #[inline]
        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            self.future.poll().map(|x| x.map(|ok| (ok,)))
        }
    }
}

// ==== EndpointAction ====

/// An enum representing the result of `EndpointAction::preflight`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Preflight<T> {
    /// The action has been completed with the specified output value.
    Completed(T),

    /// The action is incomplete.
    Incomplete,
}

impl<T> Preflight<T> {
    /// Returns `true` if the value of this is `Completed(x)`.
    pub fn is_completed(&self) -> bool {
        match self {
            Preflight::Completed(..) => true,
            Preflight::Incomplete => false,
        }
    }

    /// Returns `true` if the value of this is `InComplete`.
    pub fn is_incomplete(&self) -> bool {
        !self.is_completed()
    }

    /// Converts the contained value using the specified closure.
    pub fn map<U>(self, op: impl FnOnce(T) -> U) -> Preflight<U> {
        match self {
            Preflight::Completed(x) => Preflight::Completed(op(x)),
            Preflight::Incomplete => Preflight::Incomplete,
        }
    }
}

/// A trait that abstracts the *action* of endpoints.
pub trait EndpointAction<Bd> {
    /// The type returned from this action.
    type Output: Tuple;

    /// The error type returned from this action.
    type Error: Into<Error>;

    /// Applies an incoming request to this action and returns the result if possible.
    ///
    /// This method is called **only once** after the instance of `Self` is created.
    /// The signature of this method is almost the same as `poll_action`, but there are
    /// the following differences:
    ///
    /// * The return type of this method is **not** `Poll<Self::Output, Self::Error>`.
    ///   When this method returns an `Ok(Incomplete)`, it means that the action needs to poll
    ///   the internal resource or access `ActionContext` to complete itself. Therefore,
    ///   it is not possible to poll any asynchronous objects that this action holds
    ///   within this method.
    /// * The result of this method may affect the behavior of routing. If this method will
    ///   return an `Err`, it is possible that the combinator may ignore this error value and
    ///   choose another `EndpointAction` without aborting the process (in contrast, the error
    ///   values returned from `poll_action` are not ignored).
    ///
    /// Some limitations are added to `ApplyContext` in order to keep consistency when
    /// another endpoint returns an error (for example, it cannot be taken the instance
    /// of request body inside of this method).
    #[allow(unused_variables)]
    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        Ok(Preflight::Incomplete)
    }

    /// Progress this action and returns the result if ready.
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error>;
}

impl<F, Bd> EndpointAction<Bd> for F
where
    F: Future,
    F::Item: Tuple,
    F::Error: Into<Error>,
{
    type Output = F::Item;
    type Error = F::Error;

    fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        self.poll()
    }
}

/// Trait representing a variant of `EndpointAction` that `preflight` always returns
/// an `Ok(Preflight::Ready(x))`.
#[allow(missing_docs)]
pub trait OneshotAction {
    type Output: Tuple;
    type Error: Into<Error>;

    /// Applies an incoming request to this action and returns its result.
    fn apply(self, cx: &mut ApplyContext<'_>) -> Result<Self::Output, Self::Error>;

    /// Consume `self` and convert it into an implementor of `EndpointAction`.
    fn into_action(self) -> Oneshot<Self>
    where
        Self: Sized,
    {
        Oneshot(Some(self))
    }
}

/// Wrapper for providing an implementation of `EndpointAction` to `OneshotAction`s.
#[derive(Debug)]
pub struct Oneshot<T>(Option<T>);

impl<T, Bd> EndpointAction<Bd> for Oneshot<T>
where
    T: OneshotAction,
{
    type Output = T::Output;
    type Error = T::Error;

    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        let action = self.0.take().expect("cannot apply twice");
        action.apply(cx).map(Preflight::Completed)
    }

    fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        debug_assert!(self.0.is_none());
        unreachable!()
    }
}

/// A function to create an instance of `EndpointAction` from the specified closure.
pub fn action<Bd, T, E>(
    f: impl FnMut(&mut ActionContext<'_, Bd>) -> Poll<T, E>,
) -> impl EndpointAction<Bd, Output = T, Error = E>
where
    T: Tuple,
    E: Into<Error>,
{
    #[allow(missing_debug_implementations)]
    struct ActionFn<F>(F);

    impl<F, Bd, T, E> EndpointAction<Bd> for ActionFn<F>
    where
        F: FnMut(&mut ActionContext<'_, Bd>) -> Poll<T, E>,
        T: Tuple,
        E: Into<Error>,
    {
        type Output = T;
        type Error = E;

        fn poll_action(
            &mut self,
            cx: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            (self.0)(cx)
        }
    }

    ActionFn(f)
}

// ==== Context ====

/// A set of contextual values used by `EndpointAction::preflight`.
#[derive(Debug, Clone)]
pub struct ApplyContext<'a> {
    request: &'a Request<()>,
    pos: usize,
    popped: usize,
    _anchor: PhantomData<Rc<()>>,
}

impl<'a> ApplyContext<'a> {
    /// Creates a new `ApplyContext` with the specified reference to a `Request<()>`.
    #[inline]
    pub fn new(request: &'a Request<()>) -> Self {
        ApplyContext {
            request,
            pos: 1,
            popped: 0,
            _anchor: PhantomData,
        }
    }

    /// Returns a reference to the inner `Request<()>`.
    #[inline]
    pub fn request(&self) -> &Request<()> {
        &*self.request
    }

    /// Returns the number of segments already popped.
    pub fn num_popped_segments(&self) -> usize {
        self.popped
    }

    /// Advances the inner state and returns the next segment if possible.
    #[inline]
    pub fn next_segment(&mut self) -> Option<&'a EncodedStr> {
        let path = &self.request.uri().path();
        if self.pos == path.len() {
            return None;
        }

        let s = if let Some(offset) = path[self.pos..].find('/') {
            let s = &path[self.pos..(self.pos + offset)];
            self.pos += offset + 1;
            self.popped += 1;
            s
        } else {
            let s = &path[self.pos..];
            self.pos = path.len();
            self.popped += 1;
            s
        };

        Some(unsafe { EncodedStr::new_unchecked(s) })
    }

    /// Returns the part of remaining path that is not extracted.
    #[inline]
    pub fn remaining_path(&self) -> &'a EncodedStr {
        unsafe { EncodedStr::new_unchecked(&self.request.uri().path()[self.pos..]) }
    }
}

impl<'a> std::ops::Deref for ApplyContext<'a> {
    type Target = Request<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.request()
    }
}

impl<'a> Iterator for ApplyContext<'a> {
    type Item = &'a EncodedStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_segment()
    }
}

/// A set for contextual values used by `EndpointAction::poll_action`.
#[derive(Debug)]
pub struct ActionContext<'a, Bd> {
    request: &'a mut Request<()>,
    body: &'a mut Option<Bd>,
    _anchor: PhantomData<Rc<()>>,
}

impl<'a, Bd> ActionContext<'a, Bd> {
    /// Creates a new `ActionContext` with the specified components.
    pub fn new(request: &'a mut Request<()>, body: &'a mut Option<Bd>) -> Self {
        Self {
            request,
            body,
            _anchor: PhantomData,
        }
    }

    /// Returns a reference to the inner `Request<()>`.
    pub fn request(&self) -> &Request<()> {
        &*self.request
    }

    /// Returns a mutable reference to the inner `Request<()>`.
    pub fn request_mut(&mut self) -> &mut Request<()> {
        &mut *self.request
    }

    /// Returns a mutable reference to the instance of request body.
    pub fn body(&mut self) -> &mut Option<Bd> {
        &mut *self.body
    }
}

impl<'a, Bd> std::ops::Deref for ActionContext<'a, Bd> {
    type Target = Request<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.request()
    }
}

impl<'a, Bd> std::ops::DerefMut for ActionContext<'a, Bd> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.request_mut()
    }
}

// ==== EndpointExt =====

/// A set of extension methods for composing multiple endpoints.
pub trait EndpointExt: IsEndpoint + Sized {
    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self, E> {
        And {
            e1: self,
            e2: other,
        }
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self, E> {
        Or {
            e1: self,
            e2: other,
        }
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The differences of behaviour to `Or` are as follows:
    ///
    /// * The associated type `E::Output` must be equal to `Self::Output`.
    ///   It means that the generated endpoint has the same output type
    ///   as the original endpoints and the return value will be used later.
    /// * If `self` is matched to the request, `other.apply(cx)`
    ///   is not called and the future returned from `self.apply(cx)` is
    ///   immediately returned.
    fn or_strict<E>(self, other: E) -> OrStrict<Self, E> {
        OrStrict {
            e1: self,
            e2: other,
        }
    }

    #[allow(missing_docs)]
    fn map<F>(self, f: F) -> Map<Self, F> {
        Map { endpoint: self, f }
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self, F> {
        AndThen { endpoint: self, f }
    }

    #[allow(missing_docs)]
    fn or_else<F>(self, f: F) -> OrElse<Self, F> {
        OrElse { endpoint: self, f }
    }

    #[allow(missing_docs)]
    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        E: Into<Error>,
    {
        ErrInto {
            endpoint: self,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: IsEndpoint> EndpointExt for E {}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;

    #[test]
    fn test_segments() {
        let request = Request::get("/foo/bar.txt").body(()).unwrap();
        let mut ecx = ApplyContext::new(&request);

        assert_eq!(ecx.remaining_path(), "foo/bar.txt");
        assert_eq!(ecx.next().map(|s| s.as_bytes()), Some(&b"foo"[..]));
        assert_eq!(ecx.remaining_path(), "bar.txt");
        assert_eq!(ecx.next().map(|s| s.as_bytes()), Some(&b"bar.txt"[..]));
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next().is_none());
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next().is_none());
    }

    #[test]
    fn test_segments_from_root_path() {
        let request = Request::get("/").body(()).unwrap();
        let mut ecx = ApplyContext::new(&request);

        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next().is_none());
    }
}
