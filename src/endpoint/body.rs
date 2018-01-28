//! Components for parsing an HTTP request body.
//!
//! The key component is an endpoint `Body<T>`.
//! It will check if the incoming request is valid and start to receive
//! the request body in asynchronous mannar, finally do conversion from
//! received data into the value of `T`.
//!
//! The actual parsing of request body are in implementions of the trait
//! `FromBody`.
//! See [the documentation of `FromBody`][from_body] for details.
//!
//! If you would like to take the *raw* instance of hyper's body stream,
//! use `BodyStream` instead.
//!
//! [from_body]: ../../http/trait.FromBody.html

use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use futures::{Future, Poll};
use futures::future::{self, FutureResult};
use endpoint::{Endpoint, EndpointContext, EndpointError, EndpointResult};
use errors::BadRequest;
use http::{self, FromBody, HttpError, Request, RequestParts};

/// Creates an endpoint for parsing the incoming request body into the value of `T`
pub fn body<T: FromBody>() -> Body<T> {
    Body {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Body<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Body<T> {}

impl<T> Clone for Body<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Body<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T: FromBody> Endpoint for Body<T>
where
    T::Error: Error + 'static,
{
    type Item = T;
    type Result = BodyResult<T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match T::is_match(ctx.request()) {
            true => Some(BodyResult {
                _marker: PhantomData,
            }),
            false => None,
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BodyResult<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromBody> EndpointResult for BodyResult<T>
where
    T::Error: Error + 'static,
{
    type Item = T;
    type Future = BodyFuture<T>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let (request, body) = request.shared_parts();
        BodyFuture {
            request,
            body,
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct BodyFuture<T> {
    request: RequestParts,
    body: http::Body,
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromBody> Future for BodyFuture<T>
where
    T::Error: Error + 'static,
{
    type Item = T;
    type Error = EndpointError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let buf = try_ready!(self.body.poll());
        let body = T::from_body(&self.request, &*buf).map_err(BadRequest::new)?;
        Ok(body.into())
    }
}

/// Creates an endpoint for taking the instance of `hyper::Body`
pub fn body_stream<E>() -> BodyStream<E> {
    BodyStream {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
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

impl<E> Endpoint for BodyStream<E>
where
    E: HttpError,
{
    type Item = http::BodyStream;
    type Result = BodyStreamResult<E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(BodyStreamResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BodyStreamResult<E> {
    _marker: PhantomData<fn() -> E>,
}

impl<E> EndpointResult for BodyStreamResult<E>
where
    E: HttpError,
{
    type Item = http::BodyStream;
    type Future = FutureResult<Self::Item, EndpointError>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let body = request.body_stream().expect("cannot take a body twice");
        future::ok(body.into())
    }
}
