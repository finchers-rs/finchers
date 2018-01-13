#![allow(missing_docs)]

use std::mem;
use futures::{Async, Future, Poll};
use hyper;
use hyper::server::Service;
use endpoint::{Endpoint, EndpointResult};
use process::Process;
use responder::{IntoResponder, Responder};

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E, P>
where
    E: Endpoint,
    E::Error: IntoResponder,
    P: Process<E::Item> + Clone,
{
    endpoint: E,
    process: P,
}

impl<E, P> EndpointService<E, P>
where
    E: Endpoint,
    E::Error: IntoResponder,
    P: Process<E::Item> + Clone,
{
    pub fn new(endpoint: E, process: P) -> Self {
        EndpointService { endpoint, process }
    }
}

impl<E, P> Service for EndpointService<E, P>
where
    E: Endpoint,
    E::Error: IntoResponder,
    P: Process<E::Item> + Clone,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<<E::Result as EndpointResult>::Future, P, P::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        EndpointServiceFuture {
            inner: match self.endpoint.apply_request(req) {
                Some(input) => Inner::PollingInput(input, self.process.clone()),
                None => Inner::PollingOutput(self.process.call(None)),
            },
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<F, P, R> {
    inner: Inner<F, P, R>,
}

#[derive(Debug)]
enum Inner<F, P, R> {
    PollingInput(F, P),
    PollingOutput(R),
    Done,
}

#[allow(missing_debug_implementations)]
enum InnerError<E, P> {
    Endpoint(E),
    Process(P),
    Hyper(hyper::Error),
}

impl<F, P, R, E> Future for Inner<F, P, R>
where
    F: Future<Error = Result<E, hyper::Error>>,
    P: Process<F::Item, Future = R>,
    R: Future<Item = P::Out, Error = P::Err>,
{
    type Item = P::Out;
    type Error = InnerError<E, P::Err>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::Inner::*;
        loop {
            match mem::replace(self, Done) {
                PollingInput(mut t, p) => {
                    let input = match t.poll() {
                        Ok(Async::Ready(item)) => Some(item),
                        Ok(Async::NotReady) => {
                            *self = PollingInput(t, p);
                            break Ok(Async::NotReady);
                        }
                        Err(Ok(err)) => break Err(InnerError::Endpoint(err)),
                        Err(Err(err)) => break Err(InnerError::Hyper(err)),
                    };
                    *self = PollingOutput(p.call(input));
                    continue;
                }
                PollingOutput(mut p) => match p.poll() {
                    Ok(Async::Ready(item)) => break Ok(Async::Ready(item)),
                    Ok(Async::NotReady) => {
                        *self = PollingOutput(p);
                        break Ok(Async::NotReady);
                    }
                    Err(err) => break Err(InnerError::Process(err)),
                },
                Done => panic!(),
            }
        }
    }
}

impl<F, P, R, E> Future for EndpointServiceFuture<F, P, R>
where
    F: Future<Error = Result<E, hyper::Error>>,
    E: IntoResponder,
    P: Process<F::Item, Future = R>,
    R: Future<Item = P::Out, Error = P::Err>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(Async::Ready(item.into_responder().respond())),
            Err(InnerError::Endpoint(err)) => Ok(Async::Ready(err.into_responder().respond())),
            Err(InnerError::Process(err)) => Ok(Async::Ready(err.into_responder().respond())),
            Err(InnerError::Hyper(err)) => Err(err),
        }
    }
}
