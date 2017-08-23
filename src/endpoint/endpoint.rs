//! Definition of `Endpoint`

use futures::Future;

use super::combinator::{Map, Or, Skip, With};
use context::Context;
use errors::*;
use server::EndpointService;


/// The return type of `Endpoint::apply()`
pub type EndpointResult<T> = Result<T, EndpointError>;

/// The error type during `Endpoint::apply()`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    /// This endpoint does not matches the current request
    Skipped,
    /// The instance of requst body has already been taken
    EmptyBody,
    /// The header is not set
    EmptyHeader,
    /// The method of the current request is invalid in the endpoint
    InvalidMethod,
    /// The type of a path segment or a query parameter is not convertible to the endpoint
    TypeMismatch,
}


/// A HTTP endpoint, which provides the futures from incoming HTTP requests
pub trait Endpoint {
    /// The type of resolved value, created by this endpoint
    type Item;

    /// The type of future created by this endpoint
    type Future: Future<Item = Self::Item, Error = FinchersError>;

    /// Apply the incoming HTTP request, and return the future of its response
    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future>;


    /// Convert itself into `tokio_service::Service`
    fn into_service(self) -> EndpointService<Self>
    where
        Self: Sized,
    {
        EndpointService(self)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<E>(self, e: E) -> (Self, E)
    where
        Self: Sized,
        E: Endpoint,
    {
        (self, e)
    }

    /// Combine itself and two other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        Self: Sized,
        E1: Endpoint,
        E2: Endpoint,
    {
        (self, e1, e2)
    }

    /// Combine itself and three other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
        Self: Sized,
        E1: Endpoint,
        E2: Endpoint,
        E3: Endpoint,
    {
        (self, e1, e2, e3)
    }

    /// Combine itself and four other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> (Self, E1, E2, E3, E4)
    where
        Self: Sized,
        E1: Endpoint,
        E2: Endpoint,
        E3: Endpoint,
        E4: Endpoint,
    {
        (self, e1, e2, e3, e4)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `E::Item`.
    fn with<E>(self, e: E) -> With<Self, E>
    where
        Self: Sized,
        E: Endpoint,
    {
        With(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `Self::Item`.
    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        Self: Sized,
        E: Endpoint,
    {
        Skip(self, e)
    }

    /// Create an endpoint which attempts to apply `self`.
    /// If `self` failes, then revert the context and retry applying `e`.
    fn or<E>(self, e: E) -> Or<Self, E>
    where
        Self: Sized,
        E: Endpoint,
    {
        Or(self, e)
    }

    /// Combine itself and the function to change the return value to another type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> U,
    {
        Map(self, ::std::sync::Arc::new(f))
    }
}

impl<E: Endpoint + ?Sized> Endpoint for Box<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint + ?Sized> Endpoint for ::std::rc::Rc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint + ?Sized> Endpoint for ::std::sync::Arc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        (**self).apply(ctx)
    }
}
