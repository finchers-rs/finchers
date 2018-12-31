//! Components for constructing `Endpoint`.

mod boxed;
pub mod context;
pub mod error;
pub mod syntax;
pub mod wrapper;

mod and;
mod apply;
mod lazy;
mod or;
mod or_strict;
mod unit;
mod value;

// re-exports
pub use self::boxed::{EndpointObj, LocalEndpointObj};
pub use self::context::ApplyContext;
pub(crate) use self::context::Cursor;
pub use self::error::{ApplyError, ApplyResult};
pub use self::wrapper::{EndpointWrapExt, Wrapper};

pub use self::and::And;
pub use self::or::Or;
pub use self::or_strict::OrStrict;

pub use self::apply::{apply, apply_raw, Apply, ApplyRaw};
pub use self::lazy::{lazy, Lazy};
pub use self::unit::{unit, Unit};
pub use self::value::{value, Value};

// ====

use std::rc::Rc;
use std::sync::Arc;

use crate::common::{Combine, Tuple};
use crate::future::EndpointFuture;

/// Trait representing an endpoint.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: EndpointFuture<Output = Self::Output>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future>;

    /// Add an annotation that the associated type `Output` is fixed to `T`.
    #[inline(always)]
    fn with_output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Output = T> + Sized,
    {
        self
    }

    /// Converts `self` using the provided `Wrapper`.
    fn wrap<W>(self, wrapper: W) -> W::Endpoint
    where
        Self: Sized,
        W: Wrapper<Self>,
    {
        (wrapper.wrap(self)).with_output::<W::Output>()
    }
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (**self).apply(ecx)
    }
}

/// Trait representing the transformation into an `Endpoint`.
pub trait IntoEndpoint {
    /// The inner type of associated `Endpoint`.
    type Output: Tuple;

    /// The type of transformed `Endpoint`.
    type Endpoint: Endpoint<Output = Self::Output>;

    /// Consume itself and transform into an `Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: Endpoint> IntoEndpoint for E {
    type Output = E::Output;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

/// A set of extension methods for composing multiple endpoints.
pub trait IntoEndpointExt: IntoEndpoint + Sized {
    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self::Endpoint, E::Endpoint>
    where
        E: IntoEndpoint,
        Self::Output: Combine<E::Output>,
    {
        (And {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        })
        .with_output::<<Self::Output as Combine<E::Output>>::Out>()
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self::Endpoint, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        (Or {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        })
        .with_output::<(self::or::Wrapped<Self::Output, E::Output>,)>()
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
        E: IntoEndpoint<Output = Self::Output>,
    {
        (OrStrict {
            e1: self.into_endpoint(),
            e2: other.into_endpoint(),
        })
        .with_output::<Self::Output>()
    }
}

impl<E: IntoEndpoint> IntoEndpointExt for E {}
