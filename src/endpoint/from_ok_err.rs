#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;

use futures::{future, Future};
use endpoint::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};
use http::{Error, Request};

pub fn from_ok_err<E, A, B, C, D>(endpoint: E) -> FromOkErr<E, C, D>
where
    E: IntoEndpoint<A, B>,
    C: From<A>,
    D: From<B>,
{
    FromOkErr {
        endpoint,
        _marker: PhantomData,
    }
}

pub struct FromOkErr<E, C, D> {
    endpoint: E,
    _marker: PhantomData<fn() -> (C, D)>,
}

impl<E: Copy, C, D> Copy for FromOkErr<E, C, D> {}

impl<E: Clone, C, D> Clone for FromOkErr<E, C, D> {
    #[inline]
    fn clone(&self) -> Self {
        FromOkErr {
            endpoint: self.endpoint.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E: fmt::Debug, C, D> fmt::Debug for FromOkErr<E, C, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromOkErr")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl<E, C, D> Endpoint for FromOkErr<E, C, D>
where
    E: Endpoint,
    C: From<E::Item>,
    D: From<E::Error>,
{
    type Item = C;
    type Error = D;
    type Result = FromOkErrResult<E::Result, C, D>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(FromOkErrResult {
            result,
            _marker: PhantomData,
        })
    }
}

pub struct FromOkErrResult<R, C, D> {
    result: R,
    _marker: PhantomData<fn() -> (C, D)>,
}

impl<R: fmt::Debug, C, D> fmt::Debug for FromOkErrResult<R, C, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromOkErrResult")
            .field("result", &self.result)
            .finish()
    }
}

impl<R, C, D> EndpointResult for FromOkErrResult<R, C, D>
where
    R: EndpointResult,
    C: From<R::Item>,
    D: From<R::Error>,
{
    type Item = C;
    type Error = D;
    type Future =
        future::MapErr<future::Map<R::Future, fn(R::Item) -> C>, fn(Result<R::Error, Error>) -> Result<D, Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let future = self.result.into_future(request);
        future
            .map(Into::into as fn(R::Item) -> C)
            .map_err(|e| e.map(Into::into))
    }
}
