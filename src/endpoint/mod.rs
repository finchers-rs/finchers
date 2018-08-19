//! Components for constructing `Endpoint`.

mod and;
mod and_then;
mod boxed;
mod context;
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

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use futures_core::future::{Future, TryFuture};
use http::{Method, StatusCode};

use crate::error::{Error, HttpError};
use crate::generic::{Combine, Func, Tuple};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum EndpointErrorKind {
    NotMatched,
    MethodNotAllowed(Vec<Method>),
}

impl fmt::Display for EndpointErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndpointErrorKind::NotMatched => f.write_str("no route"),
            EndpointErrorKind::MethodNotAllowed(ref allowed_methods) => {
                if f.alternate() {
                    write!(
                        f,
                        "method not allowed (allowed methods: {:?})",
                        allowed_methods
                    )
                } else {
                    f.write_str("method not allowed")
                }
            }
        }
    }
}

impl HttpError for EndpointErrorKind {
    fn status_code(&self) -> StatusCode {
        match self {
            EndpointErrorKind::NotMatched => StatusCode::NOT_FOUND,
            EndpointErrorKind::MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}

#[allow(missing_docs)]
pub type EndpointResult<F> = Result<F, EndpointErrorKind>;

/// Trait representing an endpoint.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Output, Error = Error>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future>;
}

impl<'e, E: Endpoint> Endpoint for &'e E {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (*self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (**self).apply(ecx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
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

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: Endpoint + Sized {
    #[allow(missing_docs)]
    #[inline]
    fn output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Output = T>,
    {
        self
    }

    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint,
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
        E: IntoEndpoint,
    {
        (Or {
            e1: self,
            e2: other.into_endpoint(),
        }).output::<(self::or::WrappedEither<Self::Output, E::Output>,)>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Func<Self::Output> + Clone,
    {
        (Map { endpoint: self, f }).output::<(F::Out,)>()
    }

    #[allow(missing_docs)]
    fn then<F>(self, f: F) -> Then<Self, F>
    where
        F: Func<Self::Output> + Clone,
        F::Out: Future,
    {
        (Then { endpoint: self, f }).output::<(<F::Out as Future>::Output,)>()
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        F: Func<Self::Output> + Clone,
        F::Out: TryFuture<Error = Error>,
    {
        (AndThen { endpoint: self, f }).output::<(<F::Out as TryFuture>::Ok,)>()
    }

    #[allow(missing_docs)]
    fn boxed(self) -> Boxed<Self::Output>
    where
        Self: Send + Sync + 'static,
        Self::Future: Send + 'static,
    {
        Boxed::new(self).output::<Self::Output>()
    }

    #[allow(missing_docs)]
    fn boxed_local<'a>(self) -> BoxedLocal<'a, Self::Output>
    where
        Self: 'a,
        Self::Future: 'a,
    {
        BoxedLocal::new(self).output::<Self::Output>()
    }

    #[allow(missing_docs)]
    fn recover<F, R>(self, f: F) -> Recover<Self, F>
    where
        F: FnOnce(Error) -> R + Clone,
        R: TryFuture<Error = Error>,
    {
        (Recover { endpoint: self, f }).output::<(self::recover::Recovered<Self::Output, R::Ok>,)>()
    }

    #[allow(missing_docs)]
    fn fixed(self) -> Fixed<Self> {
        Fixed { endpoint: self }
    }
}

impl<E: Endpoint> EndpointExt for E {}
