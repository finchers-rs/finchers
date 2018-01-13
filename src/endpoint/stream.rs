#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use futures::future::{self, FutureResult};
use http::{self, Error, Request};
use super::{Endpoint, EndpointContext, EndpointResult};

pub fn body_stream<E>() -> BodyStream<E> {
    BodyStream {
        _marker: PhantomData,
    }
}

pub struct BodyStream<E> {
    _marker: PhantomData<fn() -> E>,
}

impl<E> Copy for BodyStream<E> {}

impl<E> Clone for BodyStream<E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> fmt::Debug for BodyStream<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BodyStream").finish()
    }
}

impl<E> Endpoint for BodyStream<E> {
    type Item = http::Body;
    type Error = E;
    type Result = BodyStreamResult<E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(BodyStreamResult {
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct BodyStreamResult<E> {
    _marker: PhantomData<fn() -> E>,
}

impl<E> EndpointResult for BodyStreamResult<E> {
    type Item = http::Body;
    type Error = E;
    type Future = FutureResult<Self::Item, Result<Self::Error, Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let body = request.body().expect("cannot take a body twice");
        future::ok(body)
    }
}
