//! Components of lower-level HTTP services

use futures::Async::*;
use futures::{Future, Poll};
use http::{header, Request, Response};
use std::io;
use std::sync::Arc;

use finchers_core::input::{with_input, BodyStream};
use finchers_core::output::{Body, Responder};
use finchers_core::{Error, Input};
use finchers_endpoint::{Endpoint, EndpointFuture};

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

#[allow(missing_docs)]
pub trait NewHttpService {
    type RequestBody;
    type ResponseBody;
    type Service: HttpService<RequestBody = Self::RequestBody, ResponseBody = Self::ResponseBody>;

    fn new_service(&self) -> io::Result<Self::Service>;
}

pub fn const_service<S>(service: S) -> ConstService<S> {
    ConstService {
        service: Arc::new(service),
    }
}

#[derive(Debug)]
pub struct ConstService<S> {
    service: Arc<S>,
}

impl<S> Clone for ConstService<S> {
    fn clone(&self) -> Self {
        ConstService {
            service: self.service.clone(),
        }
    }
}

impl<S: HttpService> NewHttpService for ConstService<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Service = Arc<S>;

    fn new_service(&self) -> io::Result<Self::Service> {
        Ok(self.service.clone())
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
            outcome: self.endpoint.apply_input(input),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E: Endpoint> {
    outcome: EndpointFuture<E::Future>,
}

impl<E> Future for FinchersServiceFuture<E>
where
    E: Endpoint,
    E::Item: Responder,
{
    type Item = Response<Body>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut response = match self.outcome.poll() {
            Ok(NotReady) => return Ok(NotReady),
            Ok(Ready(item)) => with_input(|input| {
                item.respond(input)
                    .unwrap_or_else(|e| Error::from(e).to_response().map(Body::once))
            }),
            Err(err) => err.to_response().map(Body::once),
        };
        if !response.headers().contains_key(header::SERVER) {
            response
                .headers_mut()
                .insert(header::SERVER, "Finchers".parse().unwrap());
        }
        Ok(Ready(response))
    }
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
    use finchers_endpoint::Endpoint;
    pub trait Sealed {}
    impl<E: Endpoint> Sealed for E {}
}
