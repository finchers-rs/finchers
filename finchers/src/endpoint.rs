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

#[derive(Debug, Clone)]
pub(crate) struct Cursor {
    pos: usize,
    popped: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor { pos: 1, popped: 0 }
    }
}

impl Cursor {
    pub(crate) fn popped(&self) -> usize {
        self.popped
    }
}

/// A marker trait indicating that the implementor has an implementation of `Endpoint<Bd>`.
pub trait IsEndpoint {}

impl<'a, E: IsEndpoint + ?Sized> IsEndpoint for &'a E {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Box<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Rc<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Arc<E> {}

/// Type alias that represents the return type of `Endpoint::apply`.
pub type Apply<Bd, E> = Result<
    <E as Endpoint<Bd>>::Action, //
    <E as Endpoint<Bd>>::Error,
>;

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

    /// Validates the incoming HTTP request and returns an instance of associated `Action` if available.
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self>;

    /// Add an annotation that the associated type `Output` is fixed to `T`.
    #[inline(always)]
    fn with_output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Bd, Output = T> + Sized,
    {
        self
    }
}

/// The contextual information during calling `Endpoint::apply`.
#[derive(Debug)]
pub struct ApplyContext<'a> {
    pub(super) request: &'a Request<()>,
    cursor: &'a mut Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> ApplyContext<'a> {
    #[inline]
    pub(crate) fn new(request: &'a Request<()>, cursor: &'a mut Cursor) -> Self {
        ApplyContext {
            request,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Returns a mutable reference to the value of `Input`.
    #[inline]
    pub fn request(&self) -> &Request<()> {
        &*self.request
    }

    pub(crate) fn cursor(&mut self) -> &mut Cursor {
        &mut *self.cursor
    }

    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &EncodedStr {
        unsafe { EncodedStr::new_unchecked(&self.request.uri().path()[self.cursor.pos..]) }
    }

    /// Advances the cursor and returns the next segment.
    #[inline]
    pub fn next_segment(&mut self) -> Option<&EncodedStr> {
        let path = &self.request.uri().path();
        if self.cursor.pos == path.len() {
            return None;
        }

        let s = if let Some(offset) = path[self.cursor.pos..].find('/') {
            let s = &path[self.cursor.pos..(self.cursor.pos + offset)];
            self.cursor.pos += offset + 1;
            self.cursor.popped += 1;
            s
        } else {
            let s = &path[self.cursor.pos..];
            self.cursor.pos = path.len();
            self.cursor.popped += 1;
            s
        };

        Some(unsafe { EncodedStr::new_unchecked(s) })
    }
}

impl<'a> std::ops::Deref for ApplyContext<'a> {
    type Target = Request<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.request
    }
}

impl<'a, E, Bd> Endpoint<Bd> for &'a E
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Box<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Rc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Arc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Action = E::Action;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        (**self).apply(ecx)
    }
}

/// Create an endpoint from a function which takes the reference to `ApplyContext`
/// and returns a future.
///
/// The endpoint created by this function will wrap the result of future into a tuple.
/// If you want to return the result without wrapping, use `apply_raw` instead.
pub fn apply_fn<Bd, R>(
    f: impl Fn(&mut ApplyContext<'_>) -> Result<R, R::Error>,
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
        F: Fn(&mut ApplyContext<'_>) -> Result<R, R::Error>,
        R: EndpointAction<Bd>,
    {
        type Output = R::Output;
        type Error = R::Error;
        type Action = R;

        #[inline]
        fn apply(&self, cx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
            (self.0)(cx)
        }
    }

    ApplyEndpoint(f)
}

/// Create an endpoint which simply returns an unit (`()`).
#[inline]
pub fn unit<Bd: 'static>() -> impl Endpoint<
    Bd, //
    Output = (),
    Error = crate::util::Never,
    Action = impl EndpointAction<Bd, Output = (), Error = crate::util::Never> + Send + 'static,
> {
    apply_fn(|_| Ok(action(|_| Ok(().into()))))
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
    Action = self::value::ValueAction<T>, // private
> {
    apply_fn(move |_| Ok(self::value::ValueAction { x: Some(x.clone()) }))
}

mod value {
    use super::*;

    // not a public API.
    #[derive(Debug)]
    pub struct ValueAction<T> {
        pub(super) x: Option<T>,
    }

    impl<T, Bd> EndpointAction<Bd> for ValueAction<T> {
        type Output = (T,);
        type Error = crate::util::Never;

        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            Ok((self.x.take().expect("The value has already taken."),).into())
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
    apply_fn(move |_| {
        Ok(self::lazy::LazyAction {
            future: f().into_future(),
        })
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

/// A trait that abstracts the *action* of endpoints, returned from `Endpoint::apply`.
pub trait EndpointAction<Bd> {
    /// The type returned from this action.
    type Output: Tuple;

    /// The error type which will be returned from this action.
    type Error: Into<Error>;

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

/// The contexual information used in the implementation of `EndpointAction::poll_action`.
#[derive(Debug)]
pub struct ActionContext<'a, Bd> {
    request: &'a Request<()>,
    body: &'a mut Option<Bd>,
    _marker: PhantomData<Rc<()>>,
}

impl<'a, Bd> ActionContext<'a, Bd> {
    pub(crate) fn new(request: &'a Request<()>, body: &'a mut Option<Bd>) -> Self {
        Self {
            request,
            body,
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    pub fn request(&self) -> &Request<()> {
        &*self.request
    }

    #[allow(missing_docs)]
    pub fn body(&mut self) -> &mut Option<Bd> {
        &mut *self.body
    }
}

impl<'a, Bd> std::ops::Deref for ActionContext<'a, Bd> {
    type Target = Request<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.request
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
        let mut cursor = Cursor::default();
        let mut ecx = ApplyContext::new(&request, &mut cursor);

        assert_eq!(ecx.remaining_path(), "foo/bar.txt");
        assert_eq!(ecx.next_segment().map(|s| s.as_bytes()), Some(&b"foo"[..]));
        assert_eq!(ecx.remaining_path(), "bar.txt");
        assert_eq!(
            ecx.next_segment().map(|s| s.as_bytes()),
            Some(&b"bar.txt"[..])
        );
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }

    #[test]
    fn test_segments_from_root_path() {
        let request = Request::get("/").body(()).unwrap();
        let mut cursor = Cursor::default();
        let mut ecx = ApplyContext::new(&request, &mut cursor);

        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }
}
