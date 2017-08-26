//! Definition of `Endpoint`

use futures::{Future, IntoFuture};

use context::Context;
use super::combinator::{and_then, map, map_err, or, or_else, skip, with, AndThen, Map, MapErr, Or, OrElse, Skip, With};
use super::result::EndpointResult;


/// A HTTP endpoint, which provides the futures from incoming HTTP requests
pub trait Endpoint {
    /// The type of resolved value, created by this endpoint
    type Item;

    #[allow(missing_docs)]
    type Error;

    /// The type of future created by this endpoint
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    /// Apply the incoming HTTP request, and return the future of its response
    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future>;


    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<E>(self, e: E) -> (Self, E)
    where
        Self: Sized,
        E: Endpoint<Error=Self::Error>,
    {
        (self, e)
    }

    /// Combine itself and two other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        Self: Sized,
        E1: Endpoint<Error=Self::Error>,
        E2: Endpoint<Error=Self::Error>,
    {
        (self, e1, e2)
    }

    /// Combine itself and three other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
        Self: Sized,
        E1: Endpoint<Error=Self::Error>,
        E2: Endpoint<Error=Self::Error>,
        E3: Endpoint<Error=Self::Error>,
    {
        (self, e1, e2, e3)
    }

    /// Combine itself and four other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> (Self, E1, E2, E3, E4)
    where
        Self: Sized,
        E1: Endpoint<Error=Self::Error>,
        E2: Endpoint<Error=Self::Error>,
        E3: Endpoint<Error=Self::Error>,
        E4: Endpoint<Error=Self::Error>,
    {
        (self, e1, e2, e3, e4)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `E::Item`.
    fn with<E>(self, e: E) -> With<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error=Self::Error>,
    {
        with(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `Self::Item`.
    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error=Self::Error>,
    {
        skip(self, e)
    }

    /// Create an endpoint which attempts to apply `self`.
    /// If `self` failes, then revert the context and retry applying `e`.
    fn or<E>(self, e: E) -> Or<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error=Self::Error>,
    {
        or(self, e)
    }

    /// Combine itself and the function to change the return value to another type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> U,
    {
        map(self, f)
    }

    /// Combine itself and the function to change the error value to another type.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> U,
    {
        map_err(self, f)
    }

    #[allow(missing_docs)]
    fn and_then<F, Fut>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> Fut,
        Fut: IntoFuture<Error = Self::Error>,
    {
        and_then(self, f)
    }

    #[allow(missing_docs)]
    fn or_else<F, Fut>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> Fut,
        Fut: IntoFuture<Item = Self::Item>,
    {
        or_else(self, f)
    }
}
