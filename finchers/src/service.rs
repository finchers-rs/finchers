//! Components of lower-level HTTP services

use std::string::ToString;
use futures::{Future, Poll};
use futures::Async::*;
use http::header;
use hyper::{self, Request, Response};
use hyper::server::Service;

use endpoint::{Endpoint, EndpointFuture, Outcome};
use response::{DefaultResponder, HttpStatus, Responder};

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

impl<E, R> Service for FinchersService<E, R>
where
    E: Endpoint,
    E::Item: Into<Outcome<R::Item>>,
    R: Responder + Clone,
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FinchersServiceFuture<E, R>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let input = ::http::Request::from(request).into();
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
    outcome: EndpointFuture<E::Result, R::Item>,
    responder: R,
}

impl<E, R> Future for FinchersServiceFuture<E, R>
where
    E: Endpoint,
    E::Item: Into<Outcome<R::Item>>,
    R: Responder,
{
    type Item = Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let outcome = try_ready!(self.outcome.poll().map_err(Into::<hyper::Error>::into));
        let mut response = self.responder.respond(outcome);
        if !response.headers().contains_key(header::SERVER) {
            response
                .headers_mut()
                .insert(header::SERVER, "Finchers".parse().unwrap());
        }
        let response = response.map(Into::into).into();
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
