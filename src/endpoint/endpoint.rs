use std::rc::Rc;
use std::sync::Arc;
use task::{IntoTask, Task};
use super::{EndpointContext, EndpointError};
use super::primitive::*;


/// A HTTP endpoint, which provides the futures from incoming HTTP requests
pub trait Endpoint {
    /// The type of resolved value, created by this endpoint
    type Item;

    #[allow(missing_docs)]
    type Error;

    /// The type of future created by this endpoint
    type Task: Task<Item = Self::Item, Error = Self::Error>;

    /// Apply the incoming HTTP request, and return the future of its response
    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError>;


    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<E>(self, e: E) -> Join<Self, E, Self::Error>
    where
        Self: Sized,
        E: Endpoint<Error = Self::Error>,
    {
        join(self, e)
    }

    #[allow(missing_docs)]
    fn join3<E1, E2>(self, e1: E1, e2: E2) -> Join3<Self, E1, E2, Self::Error>
    where
        Self: Sized,
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
    {
        join3(self, e1, e2)
    }

    #[allow(missing_docs)]
    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> Join4<Self, E1, E2, E3, Self::Error>
    where
        Self: Sized,
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
    {
        join4(self, e1, e2, e3)
    }

    #[allow(missing_docs)]
    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> Join5<Self, E1, E2, E3, E4, Self::Error>
    where
        Self: Sized,
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
        E4: Endpoint<Error = Self::Error>,
    {
        join5(self, e1, e2, e3, e4)
    }

    #[allow(missing_docs)]
    fn join6<E1, E2, E3, E4, E5>(
        self,
        e1: E1,
        e2: E2,
        e3: E3,
        e4: E4,
        e5: E5,
    ) -> Join6<Self, E1, E2, E3, E4, E5, Self::Error>
    where
        Self: Sized,
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
        E4: Endpoint<Error = Self::Error>,
        E5: Endpoint<Error = Self::Error>,
    {
        join6(self, e1, e2, e3, e4, e5)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `E::Item`.
    fn with<E>(self, e: E) -> With<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error = Self::Error>,
    {
        with(self, e)
    }

    /// Combine itself and the other endpoint, and create a combinator which returns `Self::Item`.
    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error = Self::Error>,
    {
        skip(self, e)
    }

    /// Create an endpoint which attempts to apply `self`.
    /// If `self` failes, then revert the context and retry applying `e`.
    fn or<E>(self, e: E) -> Or<Self, E>
    where
        Self: Sized,
        E: Endpoint<Item = Self::Item, Error = Self::Error>,
    {
        or(self, e)
    }

    /// Combine itself and a function to change the return value to another type.
    fn map<F, U>(self, f: F) -> Map<Self, F, U>
    where
        Self: Sized,
        F: Fn(Self::Item) -> U,
    {
        map(self, f)
    }

    /// Combine itself and a function to change the error value to another type.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F, U>
    where
        Self: Sized,
        F: Fn(Self::Error) -> U,
    {
        map_err(self, f)
    }

    #[allow(missing_docs)]
    fn and_then<F, R>(self, f: F) -> AndThen<Self, F, R>
    where
        Self: Sized,
        F: Fn(Self::Item) -> R,
        R: IntoTask<Error = Self::Error>,
    {
        and_then(self, f)
    }

    #[allow(missing_docs)]
    fn or_else<F, R>(self, f: F) -> OrElse<Self, F, R>
    where
        Self: Sized,
        F: Fn(Self::Error) -> R,
        R: IntoTask<Item = Self::Item>,
    {
        or_else(self, f)
    }

    #[allow(missing_docs)]
    fn then<F, R>(self, f: F) -> Then<Self, F, R>
    where
        Self: Sized,
        F: Fn(Result<Self::Item, Self::Error>) -> R,
        R: IntoTask,
    {
        then(self, f)
    }

    #[allow(missing_docs)]
    fn from_err<T>(self) -> FromErr<Self, T>
    where
        Self: Sized,
        T: From<Self::Error>,
    {
        from_err(self)
    }

    #[allow(missing_docs)]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Item),
    {
        inspect(self, f)
    }

    #[allow(missing_docs)]
    #[inline]
    fn with_type<T, E>(self) -> Self
    where
        Self: Sized + Endpoint<Item = T, Error = E>,
    {
        self
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
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
