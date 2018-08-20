//! Components for constructing `Endpoint`.

mod context;
mod error;

mod and;
mod and_then;
mod boxed;
mod fixed;
mod lazy;
mod map;
mod or;
mod recover;
mod reject;
mod then;
mod try_chain;
mod unit;
mod value;

// re-exports
pub use self::context::Context;
pub(crate) use self::error::AllowedMethods;
pub use self::error::{EndpointError, EndpointResult};

pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::boxed::{Boxed, BoxedLocal};
pub use self::fixed::Fixed;
pub use self::map::Map;
pub use self::or::Or;
pub use self::recover::Recover;
pub use self::then::Then;

pub use self::lazy::{lazy, Lazy};
pub use self::reject::{reject, Reject};
pub use self::unit::{unit, Unit};
pub use self::value::{value, Value};

// ====

use std::rc::Rc;
use std::sync::Arc;

use futures_core::future::{Future, TryFuture};

use crate::error::Error;
use crate::generic::{Combine, Func, Tuple};

/// Trait representing an endpoint.
pub trait Endpoint<'a>: 'a {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Output, Error = Error> + 'a;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future>;
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
pub trait EndpointExt<'a>: Endpoint<'a> + Sized {
    #[allow(missing_docs)]
    #[inline]
    fn output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<'a, Output = T>,
    {
        self
    }

    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint<'a>,
        Self::Output: Combine<E::Output>,
    {
        (And {
            e1: self,
            e2: other.into_endpoint(),
        }).output::<<Self::Output as Combine<E::Output>>::Out>()
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint<'a>,
    {
        (Or {
            e1: self,
            e2: other.into_endpoint(),
        }).output::<(self::or::WrappedEither<Self::Output, E::Output>,)>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Func<Self::Output> + 'a,
    {
        (Map { endpoint: self, f }).output::<(F::Out,)>()
    }

    #[allow(missing_docs)]
    fn then<F>(self, f: F) -> Then<Self, F>
    where
        F: Func<Self::Output> + 'a,
        F::Out: Future,
    {
        (Then { endpoint: self, f }).output::<(<F::Out as Future>::Output,)>()
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        F: Func<Self::Output> + 'a,
        F::Out: TryFuture<Error = Error>,
    {
        (AndThen { endpoint: self, f }).output::<(<F::Out as TryFuture>::Ok,)>()
    }

    #[allow(missing_docs)]
    fn recover<F, R>(self, f: F) -> Recover<Self, F>
    where
        F: Fn(Error) -> R + 'a,
        R: TryFuture<Error = Error> + 'a,
    {
        (Recover { endpoint: self, f }).output::<(self::recover::Recovered<Self::Output, R::Ok>,)>()
    }

    #[allow(missing_docs)]
    fn fixed(self) -> Fixed<Self> {
        Fixed { endpoint: self }
    }

    #[allow(missing_docs)]
    fn boxed<T: Tuple>(self) -> Boxed<T>
    where
        for<'e> Self: self::boxed::BoxedEndpoint<'e, Output = T> + Send + Sync + 'static,
    {
        Boxed::new(self).output::<T>()
    }

    #[allow(missing_docs)]
    fn boxed_local<T: Tuple>(self) -> BoxedLocal<T>
    where
        for<'e> Self: self::boxed::LocalBoxedEndpoint<'e, Output = T> + 'static,
    {
        BoxedLocal::new(self).output::<T>()
    }
}

impl<'a, E: Endpoint<'a>> EndpointExt<'a> for E {}
