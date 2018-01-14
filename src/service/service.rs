#![allow(missing_docs)]

use std::mem;
use futures::{Future, Poll};
use futures::Async::*;
use hyper;
use hyper::server::Service;
use endpoint::{Endpoint, EndpointResult};
use handler::Handler;
use responder::{self, Responder};

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    endpoint: E,
    handler: H,
    responder: R,
}

impl<E, H, R> EndpointService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    pub fn new(endpoint: E, handler: H, responder: R) -> Self {
        EndpointService {
            endpoint,
            handler,
            responder,
        }
    }
}

impl<E, H, R> Service for EndpointService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item> + Clone,
    R: Responder<H::Item, E::Error, H::Error> + Clone,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<E, H, R>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        EndpointServiceFuture {
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
pub struct EndpointServiceFuture<E, H, R>
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

impl<E, H, R> EndpointServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder<H::Item, E::Error, H::Error>,
{
    fn poll_state(&mut self) -> Poll<Result<H::Item, responder::Error<E::Error, H::Error>>, hyper::Error> {
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
                    Err(err) => break Ok(Ready(Err(responder::Error::Process(err)))),
                },
                Done => panic!(),
            }
        }
    }
}

impl<E, H, R> Future for EndpointServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item>,
    R: Responder<H::Item, E::Error, H::Error>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let input = try_ready!(self.poll_state());
        let response = self.responder.respond(input);
        Ok(Ready(response))
    }
}
