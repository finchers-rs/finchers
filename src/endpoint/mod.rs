//! Components for constructing `Endpoint`.

mod context;
pub mod error;
mod into_local;
pub mod syntax;

mod and;
mod and_then;
mod apply_fn;
mod before_apply;
mod boxed;
mod fixed;
mod lazy;
mod map;
mod or;
mod or_reject;
mod or_strict;
mod recover;
mod reject;
mod then;
mod try_chain;
mod unit;
mod value;

// re-exports
pub use self::boxed::{Boxed, BoxedLocal};
pub use self::context::Context;
pub use self::error::{EndpointError, EndpointResult};
pub use self::into_local::IntoLocal;

pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::before_apply::BeforeApply;
#[allow(deprecated)]
#[doc(hidden)]
pub use self::fixed::Fixed;
pub use self::map::Map;
pub use self::or::Or;
pub use self::or_reject::{OrReject, OrRejectWith};
pub use self::or_strict::OrStrict;
pub use self::recover::Recover;
pub use self::then::Then;

pub use self::apply_fn::{apply_fn, ApplyFn};
#[allow(deprecated)]
#[doc(hidden)]
pub use self::lazy::{lazy, Lazy};
#[allow(deprecated)]
#[doc(hidden)]
pub use self::reject::{reject, Reject};
pub use self::unit::{unit, Unit};
pub use self::value::{value, Value};

// ====

use std::rc::Rc;
use std::sync::Arc;

use futures_core::future::{Future, TryFuture};

use crate::common::{Combine, Func, Tuple};
use crate::error::Error;

/// Trait representing an endpoint.
pub trait Endpoint<'a>: 'a {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Output, Error = Error> + 'a;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future>;

    /// Add an annotation that the associated type `Output` is fixed to `T`.
    #[inline(always)]
    fn with_output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<'a, Output = T> + Sized,
    {
        self
    }

    /// Converts itself into an object which returns a `FutureObj`.
    #[inline]
    fn boxed<T: Tuple + 'static>(self) -> Boxed<T>
    where
        Self: self::boxed::IntoBoxed<T> + Sized,
    {
        (Boxed {
            inner: Box::new(self),
        }).with_output::<T>()
    }

    /// Converts itself into an object which returns a `LocalFutureObj`.
    #[inline]
    fn boxed_local<T: Tuple + 'static>(self) -> BoxedLocal<T>
    where
        Self: self::boxed::IntoBoxedLocal<T> + Sized,
    {
        (BoxedLocal {
            inner: Box::new(self),
        }).with_output::<T>()
    }
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for Box<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for Rc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for Arc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (**self).apply(ecx)
    }
}

/// A trait representing an endpoint with a constraint that the returned "Future"
/// to be transferred across thread boundaries.
pub trait SendEndpoint<'a>: 'a {
    #[allow(missing_docs)]
    type Output: Tuple;

    #[allow(missing_docs)]
    type Future: TryFuture<Ok = Self::Output, Error = Error> + Send + 'a;

    #[allow(missing_docs)]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future>;

    /// Convert itself into an representation as an `Endpoint`.
    #[inline]
    fn into_local(self) -> IntoLocal<Self>
    where
        Self: Sized,
    {
        (IntoLocal { endpoint: self }).with_output::<Self::Output>()
    }
}

impl<'a, E> SendEndpoint<'a> for E
where
    E: Endpoint<'a>,
    E::Future: Send,
{
    type Output = E::Output;
    type Future = E::Future;

    #[inline(always)]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.apply(cx)
    }
}

/// Trait representing the transformation into an `Endpoint`.
pub trait IntoEndpoint<'a> {
    /// The inner type of associated `Endpoint`.
    type Output: Tuple;

    /// The type of transformed `Endpoint`.
    type Endpoint: Endpoint<'a, Output = Self::Output>;

    /// Consume itself and transform into an `Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<'a, E: Endpoint<'a>> IntoEndpoint<'a> for E {
    type Output = E::Output;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt<'a>: IntoEndpoint<'a> + Sized {
    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.4",
        note = "use `Endpoint::with_output` instead."
    )]
    #[inline]
    fn output<T: Tuple>(self) -> Self
    where
        Self: IntoEndpoint<'a, Output = T>,
    {
        self
    }

    #[allow(missing_docs)]
    fn before_apply<F>(self, f: F) -> BeforeApply<Self::Endpoint, F>
    where
        F: Fn(&mut Context<'_>) -> EndpointResult<()> + 'a,
    {
        (BeforeApply {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<Self::Output>()
    }

    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self::Endpoint, E::Endpoint>
    where
        E: IntoEndpoint<'a>,
        Self::Output: Combine<E::Output>,
    {
        (And {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        }).with_output::<<Self::Output as Combine<E::Output>>::Out>()
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self::Endpoint, E::Endpoint>
    where
        E: IntoEndpoint<'a>,
    {
        (Or {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        }).with_output::<(self::or::Wrapped<Self::Output, E::Output>,)>()
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
    fn or_strict<E>(self, other: E) -> OrStrict<Self::Endpoint, E::Endpoint>
    where
        E: IntoEndpoint<'a, Output = Self::Output>,
    {
        (OrStrict {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        }).with_output::<Self::Output>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self::Endpoint, F>
    where
        F: Func<Self::Output> + 'a,
    {
        (Map {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<(F::Out,)>()
    }

    #[allow(missing_docs)]
    fn then<F>(self, f: F) -> Then<Self::Endpoint, F>
    where
        F: Func<Self::Output> + 'a,
        F::Out: Future + 'a,
    {
        (Then {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<(<F::Out as Future>::Output,)>()
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self::Endpoint, F>
    where
        F: Func<Self::Output> + 'a,
        F::Out: TryFuture<Error = Error> + 'a,
    {
        (AndThen {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<(<F::Out as TryFuture>::Ok,)>()
    }

    /// Creates an endpoint which returns the error value returned from
    /// `Endpoint::apply()` as the return value from the associated `Future`.
    fn or_reject(self) -> OrReject<Self::Endpoint> {
        (OrReject {
            endpoint: self.into_endpoint(),
        }).with_output::<Self::Output>()
    }

    /// Creates an endpoint which converts the error value returned from
    /// `Endpoint::apply()` to the specified type and returns it as
    /// the return value from the associated `Future`.
    fn or_reject_with<F, R>(self, f: F) -> OrRejectWith<Self::Endpoint, F>
    where
        F: Fn(EndpointError, &mut Context<'_>) -> R + 'a,
        R: Into<Error> + 'a,
    {
        (OrRejectWith {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<Self::Output>()
    }

    #[allow(missing_docs)]
    fn recover<F, R>(self, f: F) -> Recover<Self::Endpoint, F>
    where
        F: Fn(Error) -> R + 'a,
        R: TryFuture<Error = Error> + 'a,
    {
        (Recover {
            endpoint: self.into_endpoint(),
            f,
        }).with_output::<(self::recover::Recovered<Self::Output, R::Ok>,)>()
    }

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.3",
        note = "this method is going to remove before releasing 0.12.0."
    )]
    #[allow(deprecated)]
    fn fixed(self) -> Fixed<Self::Endpoint> {
        Fixed {
            endpoint: self.into_endpoint(),
        }
    }
}

impl<'a, E: IntoEndpoint<'a>> EndpointExt<'a> for E {}

#[allow(deprecated)]
mod shared {
    use futures_core::future::TryFuture;

    use super::{Context, Endpoint, EndpointResult};
    use crate::common::Tuple;
    use crate::error::Error;

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.4",
        note = "use `SendEndpoint` instead"
    )]
    pub trait SharedEndpoint<T: Tuple>: for<'a> Sealed<'a, Output = T> {
        fn into_endpoint(self) -> IntoEndpoint<Self>
        where
            Self: Sized,
        {
            IntoEndpoint(self)
        }
    }

    impl<E, T: Tuple> SharedEndpoint<T> for E where for<'a> E: Sealed<'a, Output = T> {}

    pub trait Sealed<'a>: Send + Sync + 'static {
        type Output: Tuple;
        type Future: TryFuture<Ok = Self::Output, Error = Error> + Send + 'a;

        fn apply_shared(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future>;
    }

    impl<'a, E> Sealed<'a> for E
    where
        E: Endpoint<'a> + Send + Sync + 'static,
        E::Future: Send,
    {
        type Output = E::Output;
        type Future = E::Future;

        #[inline(always)]
        fn apply_shared(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
            self.apply(cx)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct IntoEndpoint<E>(E);

    impl<'a, E: Sealed<'a>> Endpoint<'a> for IntoEndpoint<E> {
        type Output = E::Output;
        type Future = E::Future;

        #[inline(always)]
        fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
            self.apply_shared(cx)
        }
    }
}

#[doc(hidden)]
#[allow(deprecated)]
pub use self::shared::SharedEndpoint;
