//! Components of lower-level HTTP services

use futures::Async::*;
use futures::{Future, Poll};
use http::{header, Request, Response};
use std::io;
use std::sync::Arc;

use finchers_core::Input;
use finchers_core::endpoint::{Endpoint, Error};
use finchers_core::input::BodyStream;
use finchers_core::output::{Body, Responder};
use finchers_core::util::{create_task, EndpointTask};

#[allow(missing_docs)]
pub trait HttpService {
    type RequestBody;
    type ResponseBody;
    type Error;
    type Future: Future<Item = Response<Self::ResponseBody>, Error = Self::Error>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future;
}

impl<S: HttpService> HttpService for Box<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

impl<S: HttpService> HttpService for Arc<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

pub type ErrorHandler = fn(Error) -> Response<Body>;

fn default_error_handler(err: Error) -> Response<Body> {
    err.to_response().map(Body::once)
}

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Copy, Clone)]
pub struct EndpointService<E> {
    endpoint: E,
    error_handler: ErrorHandler,
}

impl<E> EndpointService<E> {
    pub fn new(endpoint: E) -> EndpointService<E> {
        Self {
            endpoint,
            error_handler: default_error_handler,
        }
    }

    pub fn set_error_handler(&mut self, handler: ErrorHandler) {
        self.error_handler = handler;
    }
}

impl<E> HttpService for EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
{
    type RequestBody = BodyStream;
    type ResponseBody = Body;
    type Error = io::Error;
    type Future = EndpointServiceFuture<E>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        let input = Input::from(request);
        EndpointServiceFuture {
            task: create_task(&self.endpoint, input),
            error_handler: self.error_handler,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<E: Endpoint> {
    task: EndpointTask<E::Task>,
    error_handler: ErrorHandler,
}

impl<E> Future for EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: Responder,
{
    type Item = Response<Body>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (result, input) = try_ready!(self.task.poll().map_err(io_error));
        let result = result.and_then(|item| item.respond(&input).map_err(Into::into));

        let mut response = result.unwrap_or_else(|err| (self.error_handler)(err));

        if !response.headers().contains_key(header::SERVER) {
            response
                .headers_mut()
                .insert(header::SERVER, "Finchers".parse().unwrap());
        }

        Ok(Ready(response))
    }
}

fn io_error<T>(_: T) -> io::Error {
    unreachable!()
}
