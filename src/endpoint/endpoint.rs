//! Definition of `Endpoint`

use std::rc::Rc;
use std::sync::Arc;
use hyper::StatusCode;

use context::Context;
use response::{IntoResponder, Responder, Response};
use task::{IntoTask, Task};

use super::combinator::*;


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

#[derive(Debug)]
pub struct EndpointErrorResponder(EndpointError);

impl Responder for EndpointErrorResponder {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }
}

impl IntoResponder for EndpointError {
    type Responder = EndpointErrorResponder;
    fn into_responder(self) -> EndpointErrorResponder {
        EndpointErrorResponder(self)
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
    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError>;


    /// Combine itself and the other endpoint, and create a combinator which returns a pair of its
    /// `Item`s.
    fn join<E>(self, e: E) -> Join<Self, E>
    where
        Self: Sized,
        E: Endpoint<Error = Self::Error>,
    {
        join(self, e)
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

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        (**self).apply(ctx)
    }
}
