//! Components for constructing `Endpoint`.

mod and;
mod and_then;
mod map;
mod ok;
mod or;
mod reject;
mod try_chain;

// re-exports
pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::map::Map;
pub use self::or::Or;

pub use self::ok::{ok, Ok};
pub use self::reject::{reject, Reject};

// ====

use std::mem::PinMut;
use std::rc::Rc;
use std::sync::Arc;

use futures_core::future::TryFuture;

use error::Error;
use generic::{Combine, Func, Tuple};
use input::{Cursor, Input};

/// Trait representing an endpoint.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Output, Error = Error>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)>;
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (*self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
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
        E: IntoEndpoint<Output = Self::Output>,
    {
        (Or {
            e1: self,
            e2: other.into_endpoint(),
        }).output::<Self::Output>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Func<Self::Output> + Clone,
    {
        (Map { endpoint: self, f }).output::<(F::Out,)>()
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        F: Func<Self::Output> + Clone,
        F::Out: TryFuture<Error = Error>,
        <F::Out as TryFuture>::Ok: Tuple,
    {
        (AndThen { endpoint: self, f }).output::<<F::Out as TryFuture>::Ok>()
    }
}

impl<E: Endpoint> EndpointExt for E {}
