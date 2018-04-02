//! Components of lower-level HTTP services

use futures::Async::*;
use futures::{Future, Poll};
use http::{header, Request, Response};
use std::io;
use std::rc::Rc;
use std::string::ToString;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointFuture, Input, Outcome};
use request::body::BodyStream;
use response::{DefaultResponder, HttpStatus, Responder, ResponseBody};

#[allow(missing_docs)]
pub trait HttpService {
    type RequestBody;
    type ResponseBody: ResponseBody;
    type Future: Future<Item = Response<Self::ResponseBody>, Error = io::Error>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future;
}

impl<S: HttpService> HttpService for Rc<S> {
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
    type ResponseBody: ResponseBody;
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
pub struct FinchersService<E, R> {
    endpoint: E,
    responder: R,
}

impl<E, R> FinchersService<E, R> {
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E, responder: R) -> Self {
        Self {
            endpoint,
            responder,
        }
    }
}

impl<E, R> HttpService for FinchersService<E, R>
where
    E: Endpoint,
    E::Item: Into<Outcome<R::Item>>,
    R: Responder + Clone,
{
    type RequestBody = BodyStream;
    type ResponseBody = R::Body;
    type Future = FinchersServiceFuture<E, R>;

    fn call(&self, request: Request<BodyStream>) -> Self::Future {
        let input = Input::from(request);
        FinchersServiceFuture {
            outcome: self.endpoint.apply_input(input),
            responder: self.responder.clone(),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E, R>
where
    E: Endpoint,
    E::Item: Into<Outcome<R::Item>>,
    R: Responder,
{
    outcome: EndpointFuture<E::Future, R::Item>,
    responder: R,
}

impl<E, R> Future for FinchersServiceFuture<E, R>
where
    E: Endpoint,
    E::Item: Into<Outcome<R::Item>>,
    R: Responder,
{
    type Item = Response<R::Body>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let outcome = try_ready!(self.outcome.poll().map_err(Into::<io::Error>::into));
        let mut response = self.responder.respond(outcome);
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
    fn into_service<T>(self) -> FinchersService<Self, DefaultResponder<T>>
    where
        Self::Item: Into<Outcome<T>>,
        T: ToString + HttpStatus,
        Self: Sized;

    fn with_responder<R>(self, responder: R) -> FinchersService<Self, R>
    where
        Self::Item: Into<Outcome<R::Item>>,
        R: Responder + Clone,
        Self: Sized;
}

impl<E: Endpoint> EndpointServiceExt for E {
    fn into_service<T>(self) -> FinchersService<Self, DefaultResponder<T>>
    where
        Self: Sized,
        E::Item: Into<Outcome<T>>,
        T: ToString + HttpStatus,
    {
        FinchersService::new(self, Default::default())
    }

    fn with_responder<R>(self, responder: R) -> FinchersService<Self, R>
    where
        Self::Item: Into<Outcome<R::Item>>,
        R: Responder + Clone,
        Self: Sized,
    {
        FinchersService::new(self, responder)
    }
}

mod sealed {
    use endpoint::Endpoint;
    pub trait Sealed {}
    impl<E: Endpoint> Sealed for E {}
}
