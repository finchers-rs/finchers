#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;

use futures::{future, Future};
use endpoint::{Endpoint, EndpointContext, EndpointError, EndpointResult, IntoEndpoint};
use errors::HttpError;
use http::Request;

pub fn from_err<E, A, B: HttpError, U>(endpoint: E) -> FromErr<E, U>
where
    E: IntoEndpoint<A, B>,
    U: From<B> + HttpError,
{
    FromErr {
        endpoint,
        _marker: PhantomData,
    }
}

pub struct FromErr<E, U> {
    endpoint: E,
    _marker: PhantomData<fn() -> U>,
}

impl<E: Copy, U> Copy for FromErr<E, U> {}

impl<E: Clone, U> Clone for FromErr<E, U> {
    #[inline]
    fn clone(&self) -> Self {
        FromErr {
            endpoint: self.endpoint.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E: fmt::Debug, U> fmt::Debug for FromErr<E, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromErr")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl<E, U: HttpError> Endpoint for FromErr<E, U>
where
    E: Endpoint,
    U: From<E::Error>,
{
    type Item = E::Item;
    type Error = U;
    type Result = FromErrResult<E::Result, U>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(FromErrResult {
            result,
            _marker: PhantomData,
        })
    }
}

pub struct FromErrResult<R, U> {
    result: R,
    _marker: PhantomData<fn() -> U>,
}

impl<R: fmt::Debug, U> fmt::Debug for FromErrResult<R, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromErrResult")
            .field("result", &self.result)
            .finish()
    }
}

impl<R, U> EndpointResult for FromErrResult<R, U>
where
    R: EndpointResult,
    U: From<R::Error> + HttpError,
{
    type Item = R::Item;
    type Error = U;
    type Future = future::MapErr<R::Future, fn(EndpointError<R::Error>) -> EndpointError<U>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let future = self.result.into_future(request);
        future.map_err(|e| match e {
            EndpointError::Endpoint(e) => EndpointError::Endpoint(e.into()),
            EndpointError::Http(e) => EndpointError::Http(e),
        })
    }
}
