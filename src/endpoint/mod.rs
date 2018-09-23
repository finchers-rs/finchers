//! Components for constructing `Endpoint`.

mod boxed;
mod context;
pub mod error;
pub mod syntax;
pub mod wrapper;

mod and;
mod apply_fn;
mod or;
mod or_strict;
mod unit;
mod value;

// re-exports
pub use self::boxed::{EndpointObj, LocalEndpointObj};
pub use self::context::Context;
pub use self::error::{EndpointError, EndpointResult};
pub use self::wrapper::{EndpointWrapExt, Wrapper};

pub use self::and::And;
pub use self::or::Or;
pub use self::or_strict::OrStrict;

pub use self::apply_fn::{apply_fn, ApplyFn};
pub use self::unit::{unit, Unit};
pub use self::value::{value, Value};

// ====

use std::rc::Rc;
use std::sync::Arc;

use futures::Future;

use common::{Combine, Tuple};
use error::Error;

/// Trait representing an endpoint.
pub trait Endpoint<'a>: 'a {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: Future<Item = Self::Output, Error = Error> + 'a;

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

    /// Converts `self` using the provided `Wrapper`.
    fn wrap<W>(self, wrapper: W) -> W::Endpoint
    where
        Self: Sized,
        W: Wrapper<'a, Self>,
    {
        (wrapper.wrap(self)).with_output::<W::Output>()
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
pub trait IsSendEndpoint<'a>: 'a + sealed_is_send_endpoint::Sealed {
    #[doc(hidden)]
    type Output: Tuple;
    #[doc(hidden)]
    type Future: Future<Item = Self::Output, Error = Error> + Send + 'a;
    #[doc(hidden)]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future>;
}

mod sealed_is_send_endpoint {
    use super::*;

    pub trait Sealed {}

    impl<'a, E> Sealed for E
    where
        E: Endpoint<'a>,
        E::Future: Send,
    {
    }
}

impl<'a, E> IsSendEndpoint<'a> for E
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

/// A wrapper struct which wraps a value whose type implements `IsSendEndpoint`
/// and provides the implementations of `Endpoint<'a>`.
#[derive(Debug, Copy, Clone)]
pub struct SendEndpoint<E> {
    endpoint: E,
}

impl<E> From<E> for SendEndpoint<E>
where
    for<'a> E: IsSendEndpoint<'a>,
{
    fn from(endpoint: E) -> Self {
        SendEndpoint { endpoint }
    }
}

impl<'a, E: IsSendEndpoint<'a>> Endpoint<'a> for SendEndpoint<E> {
    type Output = E::Output;
    type Future = E::Future;

    #[inline(always)]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.endpoint.apply(cx)
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

/// A set of extension methods for composing multiple endpoints.
pub trait IntoEndpointExt<'a>: IntoEndpoint<'a> + Sized {
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
}

impl<'a, E: IntoEndpoint<'a>> IntoEndpointExt<'a> for E {}

#[macro_export]
macro_rules! impl_endpoint {
    () => {
        $crate::endpoint::SendEndpoint<
            impl for<'a> $crate::endpoint::IsSendEndpoint<'a>
        >
    };
    (Output = $Output:ty) => {
        $crate::endpoint::SendEndpoint<
            impl for<'a> $crate::endpoint::IsSendEndpoint<'a, Output = $Output>
        >
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn return_unit() -> impl_endpoint!() {
        value(42).into()
    }

    fn return_value() -> impl_endpoint![Output = (u32,)] {
        value(42).into()
    }

    #[test]
    fn test_impl() {
        fn assert_impl(endpoint: impl for<'a> Endpoint<'a>) {
            drop(endpoint)
        }

        assert_impl(return_unit());
        assert_impl(return_value());
    }
}
