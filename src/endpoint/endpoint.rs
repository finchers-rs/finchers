use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;
use http;
use super::*;

/// Abstruction of an endpoint.
pub trait Endpoint {
    /// The type *on success*.
    type Item;

    /// The type *on failure*
    type Error;

    /// The type of returned value from `apply`.
    type Result: EndpointResult<Item = Self::Item, Error = Self::Error>;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Task` if matched.
    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result>;

    #[allow(missing_docs)]
    fn apply_request(&self, request: http::RawRequest) -> Option<<Self::Result as EndpointResult>::Future> {
        let mut request = http::Request::from(request);
        let task = try_opt!(self.apply(&mut EndpointContext::new(&request)));
        Some(task.into_future(&mut request))
    }

    #[allow(missing_docs)]
    fn join<T, E>(self, e: E) -> Join<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        join::join(self, e)
    }

    #[allow(missing_docs)]
    fn with<T, E>(self, e: E) -> With<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        with::with(self, e)
    }

    #[allow(missing_docs)]
    fn skip<T, E>(self, e: E) -> Skip<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        skip::skip(self, e)
    }

    #[allow(missing_docs)]
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<Self::Item, Self::Error>,
    {
        or::or(self, e)
    }

    #[allow(missing_docs)]
    fn map<F, U>(self, f: F) -> Map<Self, F, U>
    where
        Self: Sized,
        F: Fn(Self::Item) -> U,
    {
        map::map(self, f)
    }

    #[allow(missing_docs)]
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
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Item = E::Item;
    type Error = E::Error;
    type Result = E::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (*self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Result = E::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Result = E::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Result = E::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(ctx)
    }
}

/// Abstruction of types to be convert to an `Endpoint`.
pub trait IntoEndpoint<T, E> {
    /// The type of value returned from `into_endpoint`.
    type Endpoint: Endpoint<Item = T, Error = E>;

    /// Convert itself into `Self::Endpoint`.
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

impl<E> IntoEndpoint<(), E> for () {
    type Endpoint = EndpointOk<(), E>;

    fn into_endpoint(self) -> Self::Endpoint {
        ok(())
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
