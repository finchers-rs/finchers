#![allow(missing_docs)]

use std::fmt;
use std::mem;
use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use futures::Async::*;
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use errors::Error;
use http::Request;
use self::Chain::*;

pub fn and_then<E, F, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture,
    R::Error: Into<Error>,
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
    R: IntoFuture,
    R::Error: Into<Error>,
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
    R: IntoFuture,
    R::Error: Into<Error>,
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
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Item = R::Item;
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
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Item = R::Item;
    type Future = AndThenFuture<T::Future, F, R>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let future = self.result.into_future(request);
        AndThenFuture {
            inner: Chain::new(future, self.f),
        }
    }
}

#[derive(Debug)]
pub struct AndThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Fn(T::Item) -> R,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, R> Future for AndThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Fn(T::Item) -> R,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Item = R::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_future())),
            Err(err) => Err(err),
        })
    }
}

#[derive(Debug)]
pub enum Chain<A, B, C> {
    First(A, C),
    Second(B),
    Done,
}

impl<A, B, C> Chain<A, B, C>
where
    A: Future<Error = Error>,
    B: Future,
    B::Error: Into<Error>,
{
    pub fn new(a: A, c: C) -> Self {
        Chain::First(a, c)
    }

    pub fn poll<F>(&mut self, f: F) -> Poll<B::Item, Error>
    where
        F: FnOnce(Result<A::Item, Error>, C) -> Result<Result<B::Item, B>, Error>,
    {
        let a_result = match *self {
            First(ref mut a, ..) => match a.poll() {
                Ok(Ready(item)) => Ok(item),
                Ok(NotReady) => return Ok(NotReady),
                Err(e) => Err(e),
            },
            Second(ref mut b) => return b.poll().map_err(Into::into),
            Done => panic!("cannot poll twice"),
        };

        let data = match mem::replace(self, Done) {
            First(_, c) => c,
            _ => panic!(),
        };

        match f(a_result, data)? {
            Ok(item) => Ok(Ready(item)),
            Err(mut b) => {
                let result = b.poll().map_err(Into::into);
                *self = Second(b);
                result
            }
        }
    }
}
