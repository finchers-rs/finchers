#![allow(missing_docs)]

use std::fmt;
use std::sync::Arc;
use futures::{Future, Poll};
use http::Error;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint, Request};

pub fn map_err<E, F, R, A, B>(endpoint: E, f: F) -> MapErr<E::Endpoint, F>
where
    E: IntoEndpoint<A, B>,
    F: Fn(B) -> R,
{
    MapErr {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
    }
}

pub struct MapErr<E, F> {
    endpoint: E,
    f: Arc<F>,
}

impl<E, F, R> Clone for MapErr<E, F>
where
    E: Endpoint + Clone,
    F: Fn(E::Error) -> R,
{
    fn clone(&self) -> Self {
        MapErr {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
        }
    }
}

impl<E, F, R> fmt::Debug for MapErr<E, F>
where
    E: Endpoint + fmt::Debug,
    F: Fn(E::Error) -> R + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
}

impl<E, F, R> Endpoint for MapErr<E, F>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Result = MapErrResult<E::Result, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(MapErrResult {
            result,
            f: self.f.clone(),
        })
    }
}

#[derive(Debug)]
pub struct MapErrResult<T, F> {
    result: T,
    f: Arc<F>,
}

impl<T, F, R> EndpointResult for MapErrResult<T, F>
where
    T: EndpointResult,
    F: Fn(T::Error) -> R,
{
    type Item = T::Item;
    type Error = R;
    type Future = MapErrFuture<T::Future, F>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let fut = self.result.into_future(request);
        MapErrFuture {
            fut,
            f: Some(self.f),
        }
    }
}

#[derive(Debug)]
pub struct MapErrFuture<T, F> {
    fut: T,
    f: Option<Arc<F>>,
}

impl<T, F, E, R> Future for MapErrFuture<T, F>
where
    T: Future<Error = Result<E, Error>>,
    F: Fn(E) -> R,
{
    type Item = T::Item;
    type Error = Result<R, Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.fut.poll() {
            Ok(async) => Ok(async),
            Err(e) => {
                let f = self.f.take().expect("cannot reject twice");
                Err(e.map(|e| (*f)(e)))
            }
        }
    }
}
