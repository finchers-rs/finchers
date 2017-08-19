use futures::Future;

use combinator::core::{With, Map, Skip, Or};
use context::Context;
use errors::*;
use request::Body;
use server::EndpointService;


/// A factory of `Endpoint`.
pub trait NewEndpoint: Sized {
    type Item;
    type Future: Future<Item = Self::Item, Error = FinchersError>;
    type Endpoint: Endpoint<Item = Self::Item, Future = Self::Future>;

    fn new_endpoint(&self) -> Self::Endpoint;

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


/// A trait represents the HTTP endpoint.
pub trait Endpoint: Sized {
    type Item;
    type Future: Future<Item = Self::Item, Error = FinchersError>;

    /// Run the endpoint.
    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future>;


    fn join<E>(self, e: E) -> (Self, E)
    where
        E: Endpoint,
    {
        (self, e)
    }

    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        E1: Endpoint,
        E2: Endpoint,
    {
        (self, e1, e2)
    }

    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
        E1: Endpoint,
        E2: Endpoint,
        E3: Endpoint,
    {
        (self, e1, e2, e3)
    }

    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> (Self, E1, E2, E3, E4)
    where
        E1: Endpoint,
        E2: Endpoint,
        E3: Endpoint,
        E4: Endpoint,
    {
        (self, e1, e2, e3, e4)
    }

    fn with<E>(self, e: E) -> With<Self, E>
    where
        E: Endpoint,
    {
        With(self, e)
    }

    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        E: Endpoint,
    {
        Skip(self, e)
    }

    fn or<E>(self, e: E) -> Or<Self, E>
    where
        E: Endpoint<Item = Self::Item>,
    {
        Or(self, e)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Item) -> U,
    {
        Map(self, f)
    }
}
