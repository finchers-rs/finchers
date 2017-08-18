use futures::Future;
use context::Context;
use combinator::{With, Map, MapErr, Skip, Or};
use errors::EndpointResult;


/// A factory of `Endpoint`.
pub trait NewEndpoint {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;
    type Endpoint: Endpoint<Item = Self::Item, Error = Self::Error, Future = Self::Future>;

    fn new_endpoint(&self) -> Self::Endpoint;
}

impl<F, R> NewEndpoint for F
where
    F: Fn() -> R,
    R: Endpoint,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = R::Future;
    type Endpoint = R;

    fn new_endpoint(&self) -> Self::Endpoint {
        (*self)()
    }
}


/// A trait represents the HTTP endpoint.
pub trait Endpoint: Sized {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    /// Run the endpoint.
    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)>;


    fn join<E>(self, e: E) -> (Self, E)
    where
        E: Endpoint<Error = Self::Error>,
    {
        (self, e)
    }

    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2)
    }

    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2, e3)
    }

    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> (Self, E1, E2, E3, E4)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
        E4: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2, e3, e4)
    }

    fn with<E>(self, e: E) -> With<Self, E>
    where
        E: Endpoint<Error = Self::Error>,
    {
        With(self, e)
    }

    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        E: Endpoint<Error = Self::Error>,
    {
        Skip(self, e)
    }

    fn or<E>(self, e: E) -> Or<Self, E>
    where
        E: Endpoint<Item = Self::Item, Error = Self::Error>,
    {
        Or(self, e)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Item) -> U,
    {
        Map(self, f)
    }

    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> U,
    {
        MapErr(self, f)
    }
}

