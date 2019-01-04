//! Components for constructing `Endpoint`.

mod boxed;
pub mod context;
pub mod error;
pub mod syntax;

mod and;
mod and_then;
mod map;
mod or;
mod or_strict;

// re-exports
pub use self::boxed::{EndpointObj, LocalEndpointObj};
pub use self::context::ApplyContext;
pub(crate) use self::context::Cursor;
pub use self::error::{ApplyError, ApplyResult};

pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::map::Map;
pub use self::or::Or;
pub use self::or_strict::OrStrict;

// ====

use std::rc::Rc;
use std::sync::Arc;

use futures::IntoFuture;

use crate::common::Tuple;
use crate::error::Error;
use crate::future::EndpointFuture;

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

    /// The type of value which will be returned from `apply`.
    type Future: EndpointFuture<Bd, Output = Self::Output>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future>;

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
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Box<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Rc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E, Bd> Endpoint<Bd> for Arc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

/// Create an endpoint from a function which takes the reference to `ApplyContext`
/// and returns a future.
///
/// The endpoint created by this function will wrap the result of future into a tuple.
/// If you want to return the result without wrapping, use `apply_raw` instead.
pub fn apply_fn<Bd, R>(
    f: impl Fn(&mut ApplyContext<'_, Bd>) -> ApplyResult<R>,
) -> impl Endpoint<Bd, Output = R::Output, Future = R>
where
    R: EndpointFuture<Bd>,
    R::Output: Tuple,
{
    #[allow(missing_debug_implementations)]
    struct ApplyEndpoint<F>(F);

    impl<F> IsEndpoint for ApplyEndpoint<F> {}

    impl<F, Bd, R> Endpoint<Bd> for ApplyEndpoint<F>
    where
        F: Fn(&mut ApplyContext<'_, Bd>) -> ApplyResult<R>,
        R: EndpointFuture<Bd>,
        R::Output: Tuple,
    {
        type Output = R::Output;
        type Future = R;

        #[inline]
        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
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
    Future = impl EndpointFuture<Bd, Output = ()> + Send + 'static,
> {
    apply_fn(|_| {
        Ok(crate::future::poll_fn(|_| {
            Ok::<_, crate::error::Never>(().into())
        }))
    })
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
    Future = self::value::ValueFuture<T>, // private
> {
    apply_fn(move |_| Ok(self::value::ValueFuture { x: Some(x.clone()) }))
}

mod value {
    use crate::{
        error::Error,
        future::{Context, EndpointFuture, Poll},
    };

    // not a public API.
    #[derive(Debug)]
    pub struct ValueFuture<T> {
        pub(super) x: Option<T>,
    }

    impl<T, Bd> EndpointFuture<Bd> for ValueFuture<T> {
        type Output = (T,);

        fn poll_endpoint(&mut self, _: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
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
    Future = self::lazy::LazyFuture<R::Future>, // private
>
where
    R: IntoFuture<Error = Error>,
{
    apply_fn(move |_| {
        Ok(self::lazy::LazyFuture {
            future: f().into_future(),
        })
    })
}

mod lazy {
    use {
        crate::{
            error::Error,
            future::{Context, EndpointFuture, Poll},
        },
        futures::Future,
    };

    #[derive(Debug)]
    pub struct LazyFuture<F> {
        pub(super) future: F,
    }

    impl<F, Bd> EndpointFuture<Bd> for LazyFuture<F>
    where
        F: Future<Error = Error>,
    {
        type Output = (F::Item,);

        #[inline]
        fn poll_endpoint(&mut self, _: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            self.future.poll().map(|x| x.map(|ok| (ok,)))
        }
    }
}

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
}

impl<E: IsEndpoint> EndpointExt for E {}
