//! Components of lower-level HTTP services

use std::{io, mem};
use std::sync::Arc;
use futures::{Future, Poll};
use futures::Async::*;
use hyper::{Error, Request, Response};
use hyper::server::{NewService, Service};
use endpoint::{Endpoint, EndpointResult};
use handler::Handler;
use http::IntoResponse;
use responder::{self, Responder};

/// An HTTP service which wraps a `Endpoint`, `Handler` and `Responder`.
#[derive(Debug)]
pub struct FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    endpoint: E,
    handler: H,
    responder: R,
}

impl<E, H, R> FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E, handler: H, responder: R) -> Self {
        Self {
            endpoint,
            handler,
            responder,
        }
    }
}

impl<E, H, R> Copy for FinchersService<E, H, R>
where
    E: Endpoint + Copy,
    H: Handler<E::Item> + Copy,
    R: Responder<H::Item, E::Error, H::Error> + Copy,
{
}

impl<E, H, R> Clone for FinchersService<E, H, R>
where
    E: Endpoint + Clone,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            handler: self.handler.clone(),
            responder: self.responder.clone(),
        }
    }
}

impl<E, H, R> Service for FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = FinchersServiceFuture<E, H, R>;

    fn call(&self, req: Self::Request) -> Self::Future {
        FinchersServiceFuture {
            state: match self.endpoint.apply_request(req) {
                Some(input) => State::PollingInput {
                    input,
                    handler: self.handler.clone(),
                },
                None => State::NoRoute,
            },
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
    R: Responder<H::Item, E::Error, H::Error>,
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
        output: H::Future,
    },
    Done,
}

impl<E, H, R> FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder<H::Item, E::Error, H::Error>,
{
    fn poll_state(&mut self) -> Poll<Result<H::Item, responder::Error<E::Error, H::Error>>, Error> {
        use self::State::*;
        loop {
            match mem::replace(&mut self.state, Done) {
                NoRoute => break Ok(Ready(Err(responder::Error::NoRoute))),
                PollingInput { mut input, handler } => match input.poll() {
                    Ok(Ready(input)) => {
                        self.state = PollingOutput {
                            output: handler.call(input),
                        };
                        continue;
                    }
                    Ok(NotReady) => {
                        self.state = PollingInput { input, handler };
                        break Ok(NotReady);
                    }
                    Err(Ok(err)) => break Ok(Ready(Err(responder::Error::Endpoint(err)))),
                    Err(Err(err)) => break Err(err),
                },
                PollingOutput { mut output } => match output.poll() {
                    Ok(Ready(item)) => break Ok(Ready(Ok(item))),
                    Ok(NotReady) => {
                        self.state = PollingOutput { output };
                        break Ok(NotReady);
                    }
                    Err(err) => break Ok(Ready(Err(responder::Error::Handler(err)))),
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
    R: Responder<H::Item, E::Error, H::Error>,
{
    type Item = Response;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let input = try_ready!(self.poll_state());
        let response = self.responder.respond(input).into_response();
        Ok(Ready(response))
    }
}

#[allow(missing_docs)]
pub fn const_service<S: Service>(service: S) -> ConstService<S> {
    ConstService {
        service: Arc::new(service),
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ConstService<S: Service> {
    service: Arc<S>,
}

impl<S: Service> Clone for ConstService<S> {
    fn clone(&self) -> Self {
        ConstService {
            service: self.service.clone(),
        }
    }
}

impl<S: Service> NewService for ConstService<S> {
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Instance = Arc<S>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(self.service.clone())
    }
}
