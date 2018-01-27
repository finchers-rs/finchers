#![allow(missing_docs)]

use std::fmt;
use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use endpoint::{Endpoint, EndpointContext, EndpointResult, Request};
use http::Error;
use super::chain::Chain;

pub fn and_then<E, F, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    AndThen {
        endpoint,
        f: Arc::new(f),
    }
}

pub struct AndThen<E, F> {
    endpoint: E,
    f: Arc<F>,
}

impl<E, F, R> Clone for AndThen<E, F>
where
    E: Endpoint + Clone,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    fn clone(&self) -> Self {
        AndThen {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
        }
    }
}

impl<E, F, R> fmt::Debug for AndThen<E, F>
where
    E: Endpoint + fmt::Debug,
    F: Fn(E::Item) -> R + fmt::Debug,
    R: IntoFuture<Error = E::Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AndThen")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
}

impl<E, F, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Result = AndThenResult<E::Result, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let result = try_opt!(self.endpoint.apply(ctx));
        Some(AndThenResult {
            result,
            f: self.f.clone(),
        })
    }
}

#[derive(Debug)]
pub struct AndThenResult<T, F> {
    result: T,
    f: Arc<F>,
}

impl<T, F, R> EndpointResult for AndThenResult<T, F>
where
    T: EndpointResult,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = T::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = AndThenFuture<T::Future, F, T::Error, R>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let future = self.result.into_future(request);
        AndThenFuture {
            inner: Chain::new(future, self.f),
        }
    }
}

#[derive(Debug)]
pub struct AndThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, Error>>,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = E>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, E, R> Future for AndThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, Error>>,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = E>,
{
    type Item = R::Item;
    type Error = Result<R::Error, Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_future())),
            Err(err) => Err(err),
        })
    }
}
