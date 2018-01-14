#![allow(missing_docs)]

use std::mem;
use futures::{Future, Poll};
use futures::Async::*;
use hyper;
use hyper::server::Service;
use endpoint::{Endpoint, EndpointResult};
use process::Process;
use responder::{self, Responder};

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item> + Clone,
    R: Responder<P::Item, E::Error, P::Error> + Clone,
{
    endpoint: E,
    process: P,
    responder: R,
}

impl<E, P, R> EndpointService<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item> + Clone,
    R: Responder<P::Item, E::Error, P::Error> + Clone,
{
    pub fn new(endpoint: E, process: P, responder: R) -> Self {
        EndpointService {
            endpoint,
            process,
            responder,
        }
    }
}

impl<E, P, R> Service for EndpointService<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item> + Clone,
    R: Responder<P::Item, E::Error, P::Error> + Clone,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<E, P, R>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        EndpointServiceFuture {
            state: match self.endpoint.apply_request(req) {
                Some(input) => State::PollingInput(input, self.process.clone()),
                None => State::NoRoute,
            },
            responder: self.responder.clone(),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item>,
    R: Responder<P::Item, E::Error, P::Error>,
{
    state: State<E, P>,
    responder: R,
}

#[allow(missing_debug_implementations)]
enum State<E, P>
where
    E: Endpoint,
    P: Process<E::Item>,
{
    NoRoute,
    PollingInput(<E::Result as EndpointResult>::Future, P),
    PollingOutput(P::Future),
    Done,
}

impl<E, P, R> EndpointServiceFuture<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item>,
    R: Responder<P::Item, E::Error, P::Error>,
{
    fn poll_state(&mut self) -> Poll<Result<P::Item, responder::Error<E::Error, P::Error>>, hyper::Error> {
        use self::State::*;
        loop {
            match mem::replace(&mut self.state, Done) {
                NoRoute => break Ok(Ready(Err(responder::Error::NoRoute))),
                PollingInput(mut t, p) => {
                    let input = match t.poll() {
                        Ok(Ready(item)) => item,
                        Ok(NotReady) => {
                            self.state = PollingInput(t, p);
                            break Ok(NotReady);
                        }
                        Err(Ok(err)) => break Ok(Ready(Err(responder::Error::Endpoint(err)))),
                        Err(Err(err)) => break Err(err),
                    };
                    self.state = PollingOutput(p.call(input));
                    continue;
                }
                PollingOutput(mut p) => match p.poll() {
                    Ok(Ready(item)) => break Ok(Ready(Ok(item))),
                    Ok(NotReady) => {
                        self.state = PollingOutput(p);
                        break Ok(NotReady);
                    }
                    Err(err) => break Ok(Ready(Err(responder::Error::Process(err)))),
                },
                Done => panic!(),
            }
        }
    }
}

impl<E, P, R> Future for EndpointServiceFuture<E, P, R>
where
    E: Endpoint,
    P: Process<E::Item>,
    R: Responder<P::Item, E::Error, P::Error>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let input = try_ready!(self.poll_state());
        let response = self.responder.respond(input);
        Ok(Ready(response))
    }
}
