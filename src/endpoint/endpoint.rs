use std::rc::Rc;
use std::sync::Arc;
use futures::{future, Future, IntoFuture};
use http::Request;
use hyper;
use errors::{Error, HttpError};
use super::*;

/// Abstruction of an endpoint.
pub trait Endpoint {
    /// The type *on success*.
    type Item;

    /// The type of returned value from `apply`.
    type Result: EndpointResult<Item = Self::Item>;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Task` if matched.
    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result>;

    #[allow(missing_docs)]
    fn apply_request<R>(&self, request: R) -> Option<<Self::Result as EndpointResult>::Future>
    where
        R: Into<Request<hyper::Body>>,
    {
        let mut input = Input::from_request(request);
        self.apply(&input, &mut EndpointContext::new(&input))
            .map(|result| result.into_future(&mut input))
    }

    #[allow(missing_docs)]
    fn join<E>(self, e: E) -> Join<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint,
    {
        assert_endpoint::<_, (Self::Item, <E::Endpoint as Endpoint>::Item)>(join::join(self, e))
    }

    #[allow(missing_docs)]
    fn with<E>(self, e: E) -> With<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint,
    {
        assert_endpoint::<_, E::Item>(with::with(self, e))
    }

    #[allow(missing_docs)]
    fn skip<E>(self, e: E) -> Skip<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint,
    {
        assert_endpoint::<_, Self::Item>(skip::skip(self, e))
    }

    #[allow(missing_docs)]
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        Self: Sized,
        E: IntoEndpoint<Item = Self::Item>,
    {
        assert_endpoint::<_, Self::Item>(or::or(self, e))
    }

    #[allow(missing_docs)]
    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> T,
    {
        assert_endpoint::<_, T>(map::map(self, f))
    }

    #[allow(missing_docs)]
    fn and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> R,
        R: IntoFuture,
        R::Error: Into<Error>,
    {
        assert_endpoint::<_, R::Item>(and_then::and_then(self, f))
    }
}

#[inline]
fn assert_endpoint<E, T>(endpoint: E) -> E
where
    E: Endpoint<Item = T>,
{
    endpoint
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Item = E::Item;
    type Result = E::Result;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (*self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Result = E::Result;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Result = E::Result;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Result = E::Result;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        (**self).apply(input, ctx)
    }
}

/// Abstruction of returned value from an `Endpoint`.
pub trait EndpointResult {
    /// The type *on success*.
    type Item;

    /// The type of value returned from `launch`.
    type Future: Future<Item = Self::Item, Error = Error>;

    /// Launches itself and construct a `Future`, and then return it.
    ///
    /// This method will be called *after* the routing is completed.
    fn into_future(self, input: &mut Input) -> Self::Future;
}

impl<F: IntoFuture> EndpointResult for F
where
    F::Error: HttpError + 'static,
{
    type Item = F::Item;
    type Future = future::FromErr<F::Future, Error>;

    fn into_future(self, _: &mut Input) -> Self::Future {
        self.into_future().from_err()
    }
}

/// Abstruction of types to be convert to an `Endpoint`.
pub trait IntoEndpoint {
    /// The return type
    type Item;
    /// The type of value returned from `into_endpoint`.
    type Endpoint: Endpoint<Item = Self::Item>;

    /// Convert itself into `Self::Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: Endpoint> IntoEndpoint for E {
    type Item = E::Item;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl IntoEndpoint for () {
    type Item = ();
    type Endpoint = EndpointOk<()>;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        ok(())
    }
}

impl<E: IntoEndpoint> IntoEndpoint for Vec<E> {
    type Item = Vec<E::Item>;
    type Endpoint = JoinAll<E::Endpoint>;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        join_all(self)
    }
}

/// A shortcut of `IntoEndpoint::into_endpoint()`
#[inline]
pub fn endpoint<E: IntoEndpoint>(endpoint: E) -> E::Endpoint {
    endpoint.into_endpoint()
}
