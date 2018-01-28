#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;

use futures::{future, Future};
use endpoint::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};
use errors::HttpError;
use http::Request;

pub fn from_ok<E, A, B: HttpError, T>(endpoint: E) -> FromOk<E, T>
where
    E: IntoEndpoint<A, B>,
    T: From<A>,
{
    FromOk {
        endpoint,
        _marker: PhantomData,
    }
}

pub struct FromOk<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

impl<E: Copy, T> Copy for FromOk<E, T> {}

impl<E: Clone, T> Clone for FromOk<E, T> {
    #[inline]
    fn clone(&self) -> Self {
        FromOk {
            endpoint: self.endpoint.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E: fmt::Debug, T> fmt::Debug for FromOk<E, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromOk")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl<E, T> Endpoint for FromOk<E, T>
where
    E: Endpoint,
    T: From<E::Item>,
{
    type Item = T;
    type Error = E::Error;
    type Result = FromOkResult<E::Result, T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(FromOkResult {
            result,
            _marker: PhantomData,
        })
    }
}

pub struct FromOkResult<R, T> {
    result: R,
    _marker: PhantomData<fn() -> T>,
}

impl<R: fmt::Debug, T> fmt::Debug for FromOkResult<R, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromOkResult")
            .field("result", &self.result)
            .finish()
    }
}

impl<R, T> EndpointResult for FromOkResult<R, T>
where
    R: EndpointResult,
    T: From<R::Item>,
{
    type Item = T;
    type Error = R::Error;
    type Future = future::Map<R::Future, fn(R::Item) -> T>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let future = self.result.into_future(request);
        future.map(Into::into)
    }
}
