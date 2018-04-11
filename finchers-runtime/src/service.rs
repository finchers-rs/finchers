//! Components of lower-level HTTP services

use futures::Async::*;
use futures::{Future, Poll};
use http::{header, Request, Response};
use std::io;
use std::sync::Arc;

use finchers_core::Input;
use finchers_core::endpoint::Endpoint;
use finchers_core::input::BodyStream;
use finchers_core::output::{Body, Responder};
use finchers_core::util::{create_task, EndpointTask};

#[allow(missing_docs)]
pub trait HttpService {
    type RequestBody;
    type ResponseBody;
    type Future: Future<Item = Response<Self::ResponseBody>, Error = io::Error>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future;
}

impl<S: HttpService> HttpService for Box<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

impl<S: HttpService> HttpService for Arc<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

/// An HTTP service which wraps a `Endpoint`, `Handler` and `Responder`.
#[derive(Debug, Copy, Clone)]
pub struct FinchersService<E> {
    endpoint: E,
}

impl<E> FinchersService<E> {
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E) -> Self {
        Self { endpoint }
    }
}

impl<E> HttpService for FinchersService<E>
where
    E: Endpoint,
    E::Item: Responder,
{
    type RequestBody = BodyStream;
    type ResponseBody = Body;
    type Future = FinchersServiceFuture<E>;

    fn call(&self, request: Request<BodyStream>) -> Self::Future {
        let input = Input::from(request);
        FinchersServiceFuture {
            task: create_task(&self.endpoint, input),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E: Endpoint> {
    task: EndpointTask<E::Task>,
}

impl<E> Future for FinchersServiceFuture<E>
where
    E: Endpoint,
    E::Item: Responder,
{
    type Item = Response<Body>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (result, input) = try_ready!(self.task.poll().map_err(io_error));
        let mut response = result
            .and_then(|item| item.respond(&input).map_err(Into::into))
            .unwrap_or_else(|err| err.to_response().map(Body::once));

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

#[allow(missing_docs)]
pub trait EndpointServiceExt: Endpoint + sealed::Sealed {
    fn into_service(self) -> FinchersService<Self>
    where
        Self::Item: Responder,
        Self: Sized;
}

impl<E: Endpoint> EndpointServiceExt for E {
    fn into_service(self) -> FinchersService<Self>
    where
        Self::Item: Responder,
        Self: Sized,
    {
        FinchersService::new(self)
    }
}

mod sealed {
    use finchers_core::endpoint::Endpoint;
    pub trait Sealed {}
    impl<E: Endpoint> Sealed for E {}
}
