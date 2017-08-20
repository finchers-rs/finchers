//! Definition of `Endpoint` and `NewEndpoint`

use futures::Future;

use combinator::core::{With, Map, Skip, Or};
use context::Context;
use errors::*;
use server::EndpointService;


/// A factory of `Endpoint`
pub trait NewEndpoint: Sized {
    /// The type of resolved value of the endpoint created by this factory
    type Item;

    /// The `Future` returned by the endpoint created by this factory
    type Future: Future<Item = Self::Item, Error = FinchersError>;

    /// The `Endpoint` created by this factory
    type Endpoint: Endpoint<Item = Self::Item, Future = Self::Future>;

    /// Create and return a new endpoint.
    fn new_endpoint(&self) -> Self::Endpoint;

    /// Convert itself into `tokio_service::Service`
    fn into_service(self) -> EndpointService<Self> {
        EndpointService(self)
    }
}

impl<F, R> NewEndpoint for F
where
    F: Fn() -> R,
    R: Endpoint,
{
    type Item = R::Item;
    type Future = R::Future;
    type Endpoint = R;

    fn new_endpoint(&self) -> Self::Endpoint {
        (*self)()
    }
}

impl<E: NewEndpoint> NewEndpoint for ::std::rc::Rc<E> {
    type Item = E::Item;
    type Future = E::Future;
    type Endpoint = E::Endpoint;

    fn new_endpoint(&self) -> Self::Endpoint {
        (**self).new_endpoint()
    }
}

impl<E: NewEndpoint> NewEndpoint for ::std::sync::Arc<E> {
    type Item = E::Item;
    type Future = E::Future;
    type Endpoint = E::Endpoint;

    fn new_endpoint(&self) -> Self::Endpoint {
        (**self).new_endpoint()
    }
}


/// A HTTP endpoint, which provides the futures from incoming HTTP requests
pub trait Endpoint: Sized {
    /// The type of resolved value, created by this endpoint
    type Item;

    /// The type of future created by this endpoint
    type Future: Future<Item = Self::Item, Error = FinchersError>;

    /// Apply the incoming HTTP request, and return the future of its response
    fn apply<'r, 'b>(self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>);


    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<E>(self, e: E) -> (Self, E)
    where
        E: Endpoint,
    {
        (self, e)
    }

    /// Combine itself and two other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        E1: Endpoint,
        E2: Endpoint,
    {
        (self, e1, e2)
    }

    /// Combine itself and three other endpoints, and create a combinator which returns a tuple of its
    /// `Item`s.
    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
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
        E: Endpoint,
    {
        With(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `Self::Item`.
    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        E: Endpoint,
    {
        Skip(self, e)
    }

    /// Create an endpoint which attempts to apply `self`.
    /// If `self` failes, then revert the context and retry applying `e`.
    fn or<E>(self, e: E) -> Or<Self, E>
    where
        E: Endpoint<Item = Self::Item>,
    {
        Or(self, e)
    }

    /// Combine itself and the function to change the return value to another type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Item) -> U,
    {
        Map(self, f)
    }
}
