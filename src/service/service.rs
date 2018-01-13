#![allow(missing_docs)]

use std::io;
use std::mem;
use std::sync::Arc;

use futures::{Async, Future, Poll};
use hyper;
use hyper::server::{NewService, Service};

use endpoint::{Endpoint, Task};
use process::Process;
use responder::{IntoResponder, Responder};

/// The inner representation of `EndpointService`.
#[derive(Debug)]
struct EndpointServiceContext<E, P> {
    endpoint: E,
    process: Arc<P>,
}

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E, P>
where
    E: Endpoint,
    P: Process<E::Item, E::Error>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E, P>>,
}

impl<E, P> Service for EndpointService<E, P>
where
    E: Endpoint,
    P: Process<E::Item, E::Error>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<<E::Task as Task>::Future, P, P::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let inner = self.inner.endpoint.apply_request(req);
        EndpointServiceFuture {
            inner: Inner::PollingTask(inner, self.inner.process.clone()),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<F, P, R> {
    inner: Inner<F, P, R>,
}

impl<F, P, R, E> Future for EndpointServiceFuture<F, P, R>
where
    F: Future<Error = Result<E, hyper::Error>>,
    P: Process<F::Item, E, Future = R>,
    R: Future<Item = P::Out, Error = P::Err>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(Async::Ready(item.into_responder().respond())),
            Err(Ok(err)) => Ok(Async::Ready(err.into_responder().respond())),
            Err(Err(err)) => Err(err),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Inner<F, P, R> {
    PollingTask(Option<F>, Arc<P>),
    PollingResult(R),
    Done,
}

impl<F, P, R, E> Future for Inner<F, P, R>
where
    F: Future<Error = Result<E, hyper::Error>>,
    P: Process<F::Item, E, Future = R>,
    R: Future<Item = P::Out, Error = P::Err>,
{
    type Item = P::Out;
    type Error = Result<P::Err, hyper::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::Inner::*;
        loop {
            match mem::replace(self, Done) {
                PollingTask(t, p) => {
                    let input = match t {
                        Some(mut t) => {
                            let polled = t.poll();
                            match polled {
                                Ok(Async::Ready(item)) => Some(Ok(item)),
                                Ok(Async::NotReady) => {
                                    *self = PollingTask(Some(t), p);
                                    break Ok(Async::NotReady);
                                }
                                Err(Ok(err)) => Some(Err(err)),
                                Err(Err(err)) => break Err(Err(err)),
                            }
                        }
                        None => None,
                    };
                    *self = PollingResult(p.call(input));
                    continue;
                }
                PollingResult(mut p) => {
                    let polled = p.poll();
                    match polled {
                        Ok(Async::Ready(item)) => break Ok(Async::Ready(item)),
                        Ok(Async::NotReady) => {
                            *self = PollingResult(p);
                            break Ok(Async::NotReady);
                        }
                        Err(err) => break Err(Ok(err)),
                    }
                }
                Done => panic!(),
            }
        }
    }
}

#[derive(Debug)]
pub struct EndpointServiceFactory<E, P>
where
    E: Endpoint,
    P: Process<E::Item, E::Error>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E, P>>,
}

impl<E, P> EndpointServiceFactory<E, P>
where
    E: Endpoint,
    P: Process<E::Item, E::Error>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    pub fn new(endpoint: E, process: P) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                process: Arc::new(process),
            }),
        }
    }
}

impl<E, P> NewService for EndpointServiceFactory<E, P>
where
    E: Endpoint,
    P: Process<E::Item, E::Error>,
    P::Out: IntoResponder,
    P::Err: IntoResponder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = EndpointService<E, P>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(EndpointService {
            inner: self.inner.clone(),
        })
    }
}
