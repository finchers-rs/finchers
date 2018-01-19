#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;

use futures::{future, Future};
use endpoint::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};
use http::{Error, Request};

pub fn adapt<E, A, B, C, D>(endpoint: E) -> Adapt<E, C, D>
where
    E: IntoEndpoint<A, B>,
    C: From<A>,
    D: From<B>,
{
    Adapt {
        endpoint,
        _marker: PhantomData,
    }
}

pub struct Adapt<E, C, D> {
    endpoint: E,
    _marker: PhantomData<fn() -> (C, D)>,
}

impl<E: Copy, C, D> Copy for Adapt<E, C, D> {}

impl<E: Clone, C, D> Clone for Adapt<E, C, D> {
    #[inline]
    fn clone(&self) -> Self {
        Adapt {
            endpoint: self.endpoint.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E: fmt::Debug, C, D> fmt::Debug for Adapt<E, C, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Adapt")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl<E, C, D> Endpoint for Adapt<E, C, D>
where
    E: Endpoint,
    C: From<E::Item>,
    D: From<E::Error>,
{
    type Item = C;
    type Error = D;
    type Result = AdaptResult<E::Result, C, D>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(AdaptResult {
            result,
            _marker: PhantomData,
        })
    }
}

pub struct AdaptResult<R, C, D> {
    result: R,
    _marker: PhantomData<fn() -> (C, D)>,
}

impl<R: fmt::Debug, C, D> fmt::Debug for AdaptResult<R, C, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AdaptResult")
            .field("result", &self.result)
            .finish()
    }
}

impl<R, C, D> EndpointResult for AdaptResult<R, C, D>
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
