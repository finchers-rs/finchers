//! Components of lower-level HTTP services

use std::mem;
use futures::{Future, IntoFuture, Poll};
use futures::Async::*;
use hyper::{self, Request, Response};
use hyper::server::Service;

use endpoint::{Endpoint, EndpointResult};
use errors::Error;
use http::IntoResponse;
use handler::{DefaultHandler, Handler};
use responder::{DefaultResponder, Responder};

/// An HTTP service which wraps a `Endpoint`, `Handler` and `Responder`.
#[derive(Debug, Copy, Clone)]
pub struct FinchersService<E, H, R> {
    endpoint: E,
    handler: H,
    responder: R,
}

impl<E, H, R> FinchersService<E, H, R> {
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E, handler: H, responder: R) -> Self {
        Self {
            endpoint,
            handler,
            responder,
        }
    }
}

impl<E, H, R> Service for FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<Item = H::Item> + Clone,
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FinchersServiceFuture<E, H, R>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let state = match self.endpoint.apply_request(req) {
            Some(input) => State::PollingInput {
                input,
                handler: self.handler.clone(),
            },
            None => State::NoRoute,
        };
        FinchersServiceFuture {
            state,
            responder: self.responder.clone(),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder,
{
    state: State<E, H>,
    responder: R,
}

#[allow(missing_debug_implementations)]
enum State<E, H>
where
    E: Endpoint,
    H: Handler<E::Item>,
{
    NoRoute,
    PollingInput {
        input: <E::Result as EndpointResult>::Future,
        handler: H,
    },
    PollingOutput {
        output: <H::Result as IntoFuture>::Future,
    },
    Done,
}

impl<E, H, R> FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder,
{
    fn poll_state(&mut self) -> Poll<Option<H::Item>, Error> {
        use self::State::*;
        loop {
            match mem::replace(&mut self.state, Done) {
                NoRoute => break Ok(Ready(None)),
                PollingInput { mut input, handler } => match input.poll()? {
                    Ready(input) => {
                        self.state = PollingOutput {
                            output: IntoFuture::into_future(handler.call(input)),
                        };
                        continue;
                    }
                    NotReady => {
                        self.state = PollingInput { input, handler };
                        break Ok(NotReady);
                    }
                },
                PollingOutput { mut output } => match output.poll()? {
                    Ready(item) => break Ok(Ready(item)),
                    NotReady => {
                        self.state = PollingOutput { output };
                        break Ok(NotReady);
                    }
                },
                Done => panic!(),
            }
        }
    }
}

impl<E, H, R> Future for FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder<Item = H::Item>,
{
    type Item = Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut response = match self.poll_state() {
            Ok(NotReady) => return Ok(NotReady),
            Ok(Ready(Some(item))) => self.responder.respond_ok(item),
            Ok(Ready(None)) => self.responder.respond_noroute(),
            Err(err) => self.responder.respond_err(&*err),
        };
        self.responder.after_respond(&mut response);
        Ok(Ready(response))
    }
}

#[allow(missing_docs)]
pub trait EndpointServiceExt: Endpoint + sealed::Sealed
where
    Self::Item: IntoResponse,
{
    fn into_service(self) -> FinchersService<Self, DefaultHandler, DefaultResponder<Self::Item>>
    where
        Self: Sized;

    fn with_handler<H>(self, handler: H) -> FinchersService<Self, H, DefaultResponder<H::Item>>
    where
        H: Handler<Self::Item> + Clone,
        Self: Sized;
}

impl<E: Endpoint> EndpointServiceExt for E
where
    E::Item: IntoResponse,
{
    fn into_service(self) -> FinchersService<Self, DefaultHandler, DefaultResponder<Self::Item>> {
        FinchersService::new(self, DefaultHandler::default(), Default::default())
    }

    fn with_handler<H>(self, handler: H) -> FinchersService<Self, H, DefaultResponder<H::Item>>
    where
        H: Handler<Self::Item> + Clone,
    {
        FinchersService::new(self, handler, Default::default())
    }
}

mod sealed {
    use endpoint::Endpoint;
    pub trait Sealed {}
    impl<E: Endpoint> Sealed for E {}
}
