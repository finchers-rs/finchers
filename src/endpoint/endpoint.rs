use std::rc::Rc;
use std::sync::Arc;
use futures::{future, Future, IntoFuture};
use errors::HttpError;
use http::{self, Request};
use super::*;

/// Abstruction of an endpoint.
pub trait Endpoint {
    /// The type *on success*.
    type Item;

    /// The type *on failure*
    type Error: HttpError;

    /// The type of returned value from `apply`.
    type Result: EndpointResult<Item = Self::Item, Error = Self::Error>;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Task` if matched.
    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result>;

    #[allow(missing_docs)]
    fn apply_request<R: Into<Request>>(&self, request: R) -> Option<<Self::Result as EndpointResult>::Future> {
        let mut request = request.into();
        self.apply(&mut EndpointContext::new(&request))
            .map(|result| result.into_future(&mut request))
    }

    /// Add an assertion to associated types in this endpoint.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use finchers::{Endpoint, IntoEndpoint};
    /// // The error type of `endpoint` is unknown
    /// let endpoint = IntoEndpoint::into_endpoint("foo");
    ///
    /// // Add an assertion that the error type of `endpoint` must be ().
    /// let endpoint = endpoint.assert_types::<_, ()>();
    /// ```
    #[inline]
    fn assert_types<T, E>(self) -> Self
    where
        Self: Sized + Endpoint<Item = T, Error = E>,
    {
        self
    }

    #[allow(missing_docs)]
    fn join<T, E>(self, e: E) -> Join<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        join::join(self, e).assert_types::<(Self::Item, <E::Endpoint as Endpoint>::Item), Self::Error>()
    }

    #[allow(missing_docs)]
    fn with<T, E>(self, e: E) -> With<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        with::with(self, e).assert_types::<<E::Endpoint as Endpoint>::Item, Self::Error>()
    }

    #[allow(missing_docs)]
    fn skip<T, E>(self, e: E) -> Skip<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<T, Self::Error>,
    {
        skip::skip(self, e).assert_types::<Self::Item, Self::Error>()
    }

    #[allow(missing_docs)]
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<Self::Item, Self::Error>,
    {
        or::or(self, e).assert_types::<Self::Item, Self::Error>()
    }

    #[allow(missing_docs)]
    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> T,
    {
        map::map(self, f).assert_types::<T, Self::Error>()
    }

    #[allow(missing_docs)]
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> U,
        U: HttpError,
    {
        map_err::map_err(self, f).assert_types::<Self::Item, U>()
    }

    #[allow(missing_docs)]
    fn and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> R,
        R: IntoFuture<Error = Self::Error>,
    {
        and_then::and_then(self, f).assert_types::<R::Item, Self::Error>()
    }

    #[allow(missing_docs)]
    fn from_ok_err<T, E>(self) -> FromOkErr<Self, T, E>
    where
        Self: Sized,
        T: From<Self::Item>,
        E: From<Self::Error> + HttpError,
    {
        from_ok_err::from_ok_err(self).assert_types::<T, E>()
    }

    #[allow(missing_docs)]
    fn from_ok<T>(self) -> FromOk<Self, T>
    where
        Self: Sized,
        T: From<Self::Item>,
    {
        from_ok::from_ok(self).assert_types::<T, Self::Error>()
    }

    #[allow(missing_docs)]
    fn from_err<E>(self) -> FromErr<Self, E>
    where
        Self: Sized,
        E: From<Self::Error> + HttpError,
    {
        from_err::from_err(self).assert_types::<Self::Item, E>()
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

#[allow(missing_docs)]
#[derive(Debug)]
pub enum EndpointError<E: HttpError> {
    Endpoint(E),
    Http(http::Error),
}

impl<E: HttpError> From<E> for EndpointError<E> {
    fn from(err: E) -> Self {
        EndpointError::Endpoint(err)
    }
}

impl<E: HttpError> From<http::Error> for EndpointError<E> {
    fn from(err: http::Error) -> Self {
        EndpointError::Http(err)
    }
}

/// Abstruction of returned value from an `Endpoint`.
pub trait EndpointResult {
    /// The type *on success*.
    type Item;

    /// The type *on failure*.
    type Error: HttpError;

    /// The type of value returned from `launch`.
    type Future: Future<Item = Self::Item, Error = EndpointError<Self::Error>>;

    /// Launches itself and construct a `Future`, and then return it.
    ///
    /// This method will be called *after* the routing is completed.
    fn into_future(self, request: &mut Request) -> Self::Future;
}

impl<F: IntoFuture> EndpointResult for F
where
    F::Error: HttpError,
{
    type Item = F::Item;
    type Error = F::Error;
    type Future = future::MapErr<F::Future, fn(F::Error) -> EndpointError<F::Error>>;

    fn into_future(self, _: &mut Request) -> Self::Future {
        self.into_future().map_err(Into::into)
    }
}

/// Abstruction of types to be convert to an `Endpoint`.
pub trait IntoEndpoint<T, E: HttpError> {
    /// The type of value returned from `into_endpoint`.
    type Endpoint: Endpoint<Item = T, Error = E>;

    /// Convert itself into `Self::Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E, A, B: HttpError> IntoEndpoint<A, B> for E
where
    E: Endpoint<Item = A, Error = B>,
{
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<E: HttpError> IntoEndpoint<(), E> for () {
    type Endpoint = EndpointOk<(), E>;

    fn into_endpoint(self) -> Self::Endpoint {
        ok(())
    }
}

impl<T, A, B: HttpError> IntoEndpoint<Vec<A>, B> for Vec<T>
where
    T: IntoEndpoint<A, B>,
{
    type Endpoint = JoinAll<T::Endpoint>;

    fn into_endpoint(self) -> Self::Endpoint {
        join_all(self)
    }
}

/// A shortcut of `IntoEndpoint::into_endpoint()`
pub fn endpoint<E, A, B: HttpError>(endpoint: E) -> E::Endpoint
where
    E: IntoEndpoint<A, B>,
{
    endpoint.into_endpoint()
}
