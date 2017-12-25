use std::fmt;
use std::error;
use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;
use tokio_core::reactor::Handle;
use response::IntoResponder;
use task::Task;
use service::EndpointService;
use super::*;


#[allow(missing_docs)]
#[derive(Debug)]
pub struct NotFound;

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("not found")
    }
}

impl error::Error for NotFound {
    fn description(&self) -> &str {
        "not found"
    }
}


/// A HTTP endpoint, which provides the futures from incoming HTTP requests
pub trait Endpoint {
    /// The type of resolved value, created by this endpoint
    type Item;

    #[allow(missing_docs)]
    type Error;

    /// The type of future created by this endpoint
    type Task: Task<Item = Self::Item, Error = Self::Error>;

    /// Apply the incoming HTTP request, and return the future of its response
    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task>;


    /// Create a new `Service` from this endpoint
    fn to_service(&self, handle: &Handle) -> EndpointService<Self>
    where
        Self: Clone,
        Self::Item: IntoResponder,
        Self::Error: IntoResponder + From<NotFound>,
    {
        EndpointService::new(self.clone(), handle)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<T, E>(self, e: E) -> Join<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        join::join(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `E::Item`.
    fn with<T, E>(self, e: E) -> With<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        with::with(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `Self::Item`.
    fn skip<T, E>(self, e: E) -> Skip<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        skip::skip(self, e)
    }

    /// Create an endpoint which attempts to apply `self`.
    /// If `self` failes, then revert the context and retry applying `e`.
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<Self::Item, Self::Error>,
    {
        or::or(self, e)
    }

    /// Combine itself and a function to change the return value to another type.
    fn map<F, U>(self, f: F) -> Map<Self, F, U>
    where
        Self: Sized,
        F: Fn(Self::Item) -> U,
    {
        map::map(self, f)
    }

    /// Combine itself and a function to change the error value to another type.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F, U>
    where
        Self: Sized,
        F: Fn(Self::Error) -> U,
    {
        map_err::map_err(self, f)
    }

    #[allow(missing_docs)]
    fn and_then<F, R>(self, f: F) -> AndThen<Self, F, R>
    where
        Self: Sized,
        F: Fn(Self::Item) -> R,
        R: IntoFuture<Error = Self::Error>,
    {
        and_then::and_then(self, f)
    }

    #[allow(missing_docs)]
    fn or_else<F, R>(self, f: F) -> OrElse<Self, F, R>
    where
        Self: Sized,
        F: Fn(Self::Error) -> R,
        R: IntoFuture<Item = Self::Item>,
    {
        or_else::or_else(self, f)
    }

    #[allow(missing_docs)]
    fn then<F, R>(self, f: F) -> Then<Self, F, R>
    where
        Self: Sized,
        F: Fn(Result<Self::Item, Self::Error>) -> R,
        R: IntoFuture,
    {
        then::then(self, f)
    }

    #[allow(missing_docs)]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Item),
    {
        inspect::inspect(self, f)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        (**self).apply(ctx)
    }
}


#[allow(missing_docs)]
pub trait IntoEndpoint<T, E> {
    type Endpoint: Endpoint<Item = T, Error = E>;

    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E, A, B> IntoEndpoint<A, B> for E
where
    E: Endpoint<Item = A, Error = B>,
{
    type Endpoint = E;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<T, A, B> IntoEndpoint<Vec<A>, B> for Vec<T>
where
    T: IntoEndpoint<A, B>,
{
    type Endpoint = JoinAll<T::Endpoint>;

    fn into_endpoint(self) -> Self::Endpoint {
        join_all(self)
    }
}
