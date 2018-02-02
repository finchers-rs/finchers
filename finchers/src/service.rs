//! Components of lower-level HTTP services

use std::io;
use std::string::ToString;
use std::marker::PhantomData;
use futures::{Future, Poll};
use futures::Async::*;
use http::header;
use hyper::{self, Request, Response};
use hyper::server::Service;

use endpoint::{Endpoint, EndpointResult, Outcome};
use response::{DefaultResponder, HttpStatus, Responder};

/// An HTTP service which wraps a `Endpoint`, `Handler` and `Responder`.
#[derive(Debug, Copy, Clone)]
pub struct FinchersService<E, R, T> {
    endpoint: E,
    responder: R,
    _marker: PhantomData<fn() -> T>,
}

impl<E, R, T> FinchersService<E, R, T> {
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E, responder: R) -> Self {
        Self {
            endpoint,
            responder,
            _marker: PhantomData,
        }
    }
}

impl<E, R, T> Service for FinchersService<E, R, T>
where
    E: Endpoint,
    E::Item: Into<Outcome<T>>,
    R: Responder<T> + Clone,
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FinchersServiceFuture<E, R, T>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let input = ::http::Request::from(request);
        FinchersServiceFuture {
            state: self.endpoint.apply_input(input.into()),
            responder: self.responder.clone(),
            _marker: PhantomData,
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E, R, T>
where
    E: Endpoint,
    E::Item: Into<Outcome<T>>,
    R: Responder<T>,
{
    state: Option<<E::Result as EndpointResult>::Future>,
    responder: R,
    _marker: PhantomData<fn() -> T>,
}

impl<E, R, T> FinchersServiceFuture<E, R, T>
where
    E: Endpoint,
    E::Item: Into<Outcome<T>>,
    R: Responder<T>,
{
    fn poll_state(&mut self) -> Poll<Outcome<T>, io::Error> {
        let outcome = match self.state {
            Some(ref mut f) => match f.poll() {
                Ok(Ready(outcome)) => outcome.into(),
                Ok(NotReady) => return Ok(NotReady),
                Err(err) => Outcome::Err(err),
            },
            None => Outcome::NoRoute,
        };
        Ok(Ready(outcome))
    }
}

impl<E, R, T> Future for FinchersServiceFuture<E, R, T>
where
    E: Endpoint,
    E::Item: Into<Outcome<T>>,
    R: Responder<T>,
{
    type Item = Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let output = try_ready!(self.poll_state());
        let mut response = self.responder.respond(output);
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
pub trait EndpointServiceExt<T>: Endpoint + sealed::Sealed {
    fn into_service(self) -> FinchersService<Self, DefaultResponder, T>
    where
        Self::Item: Into<Outcome<T>>,
        T: ToString + HttpStatus,
        Self: Sized;

    fn with_responder<R>(self, responder: R) -> FinchersService<Self, R, T>
    where
        Self::Item: Into<Outcome<T>>,
        R: Responder<T> + Clone,
        Self: Sized;
}

impl<E: Endpoint, T> EndpointServiceExt<T> for E
where
    E::Item: Into<Outcome<T>>,
{
    fn into_service(self) -> FinchersService<Self, DefaultResponder, T>
    where
        Self: Sized,
        T: ToString + HttpStatus,
    {
        FinchersService::new(self, Default::default())
    }

    fn with_responder<R>(self, responder: R) -> FinchersService<Self, R, T>
    where
        Self::Item: Into<Outcome<T>>,
        R: Responder<T> + Clone,
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
