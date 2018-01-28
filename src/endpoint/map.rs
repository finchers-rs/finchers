#![allow(missing_docs)]

use std::fmt;
use std::sync::Arc;
use futures::{Future, Poll};
use http::Request;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};

pub fn map<E, F, R>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: Fn(E::Item) -> R,
{
    Map {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
    }
}

pub struct Map<E, F> {
    endpoint: E,
    f: Arc<F>,
}

impl<E, F, R> Clone for Map<E, F>
where
    E: Endpoint + Clone,
    F: Fn(E::Item) -> R,
{
    fn clone(&self) -> Self {
        Map {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
        }
    }
}

impl<E, F, R> fmt::Debug for Map<E, F>
where
    E: Endpoint + fmt::Debug,
    F: Fn(E::Item) -> R + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Map")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
}

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Result = MapResult<E::Result, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(MapResult {
            result,
            f: self.f.clone(),
        })
    }
}

#[derive(Debug)]
pub struct MapResult<T, F> {
    result: T,
    f: Arc<F>,
}

impl<T, F, R> EndpointResult for MapResult<T, F>
where
    T: EndpointResult,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Future = MapFuture<T::Future, F>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let fut = self.result.into_future(request);
        MapFuture {
            fut,
            f: Some(self.f),
        }
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    fut: T,
    f: Option<Arc<F>>,
}

impl<T, F, R> Future for MapFuture<T, F>
where
    T: Future,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        let f = self.f.take().expect("cannot resolve twice");
        Ok((*f)(item).into())
    }
}
