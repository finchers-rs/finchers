#![allow(missing_docs)]

use std::io;
use std::mem;
use std::sync::Arc;

use futures::{Async, Future, Poll};
use hyper;
use hyper::server::Service;
use tokio_core::reactor::Handle;

use http::{self, Cookies, SecretKey};
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use process::Process;
use responder::{IntoResponder, Responder};
use responder::ResponderContext;
use super::ServiceFactory;

/// The inner representation of `EndpointService`.
#[derive(Debug)]
struct EndpointServiceContext<E, P> {
    endpoint: E,
    process: Arc<P>,
    secret_key: SecretKey,
}

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E, P>
where
    E: Endpoint,
    P: Process<In = E::Item, InErr = E::Error>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E, P>>,
    handle: Handle,
}

impl<E, P> Service for EndpointService<E, P>
where
    E: Endpoint,
    P: Process<In = E::Item, InErr = E::Error>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<<E::Task as Task>::Future, P>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (mut request, body) = http::request::reconstruct(req);
        let mut cookies = Cookies::from_original(request.header(), self.inner.secret_key.clone());

        let task = {
            let mut ctx = EndpointContext::new(&request, &cookies);
            self.inner.endpoint.apply(&mut ctx)
        };

        let inner = task.map(|task| {
            let mut ctx = TaskContext {
                request: &mut request,
                handle: &self.handle,
                cookies: &mut cookies,
                body: Some(body),
            };
            task.launch(&mut ctx)
        });

        EndpointServiceFuture {
            inner: Inner::PollingTask(inner, self.inner.process.clone()),
            context: ResponderContext { request, cookies },
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<F, P: Process> {
    inner: Inner<F, P>,
    context: ResponderContext,
}

impl<F, E, P> Future for EndpointServiceFuture<F, P>
where
    F: Future<Error = Result<E, hyper::Error>>,
    P: Process<In = F::Item, InErr = E>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(Async::Ready(
                item.into_responder().respond(&mut self.context),
            )),
            Err(Ok(err)) => Ok(Async::Ready(
                err.into_responder().respond(&mut self.context),
            )),
            Err(Err(err)) => Err(err),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Inner<F, P: Process> {
    PollingTask(Option<F>, Arc<P>),
    PollingResult(P::Future),
    Done,
}

impl<F, P, E> Future for Inner<F, P>
where
    F: Future<Error = Result<E, hyper::Error>>,
    P: Process<In = F::Item, InErr = E>,
{
    type Item = <P::Future as Future>::Item;
    type Error = Result<<P::Future as Future>::Error, hyper::Error>;

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
                                    return Ok(Async::NotReady);
                                }
                                Err(Ok(err)) => Some(Err(err)),
                                Err(Err(err)) => return Err(Err(err)),
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
                            return Ok(Async::NotReady);
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
    P: Process<In = E::Item, InErr = E::Error>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E, P>>,
}

impl<E, P> EndpointServiceFactory<E, P>
where
    E: Endpoint,
    P: Process<In = E::Item, InErr = E::Error>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    pub fn new(endpoint: E, process: P) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                process: Arc::new(process),
                secret_key: SecretKey::generated(),
            }),
        }
    }

    #[cfg(feature = "secure")]
    pub fn set_secret_key(&mut self, key: SecretKey) {
        self.inner_mut().secret_key = key;
    }

    #[allow(dead_code)]
    fn inner_mut(&mut self) -> &mut EndpointServiceContext<E, P> {
        Arc::get_mut(&mut self.inner)
            .expect("cannot get a mutable reference of inner context of EndpointServiceFactory")
    }
}

impl<E, P> ServiceFactory for EndpointServiceFactory<E, P>
where
    E: Endpoint,
    P: Process<In = E::Item, InErr = E::Error>,
    P::Out: IntoResponder,
    P::OutErr: IntoResponder,
{
    type Service = EndpointService<E, P>;

    fn new_service(&self, handle: &Handle) -> io::Result<Self::Service> {
        Ok(EndpointService {
            inner: self.inner.clone(),
            handle: handle.clone(),
        })
    }
}
