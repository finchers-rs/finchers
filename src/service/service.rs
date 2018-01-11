#![allow(missing_docs)]

use std::io;
use std::mem;
use std::sync::Arc;

use futures::{Async, Future, IntoFuture, Poll};
use hyper;
use hyper::server::Service;
use tokio_core::reactor::Handle;

use http::{self, Cookies, SecretKey, StatusCode};
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use responder::{IntoResponder, Responder};
use responder::ResponderContext;
use super::ServiceFactory;

/// The inner representation of `EndpointService`.
#[derive(Debug)]
struct EndpointServiceContext<E> {
    endpoint: E,
    secret_key: SecretKey,
    no_route: fn() -> hyper::Response,
}

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E>>,
    handle: Handle,
}

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<<E::Task as Task>::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (mut request, body) = http::request::reconstruct(req);
        let mut cookies = Cookies::from_original(request.header(), self.inner.secret_key.clone());

        let task = {
            let mut ctx = EndpointContext::new(&request, &cookies);
            self.inner.endpoint.apply(&mut ctx)
        };

        let inner = match task {
            Some(task) => {
                let mut ctx = TaskContext {
                    request: &mut request,
                    handle: &self.handle,
                    cookies: &mut cookies,
                    body: Some(body),
                };
                Respondable::Polling(task.launch(&mut ctx).into_future())
            }
            None => Respondable::NoRoute,
        };

        EndpointServiceFuture {
            inner,
            context: ResponderContext { request, cookies },
            no_route: self.inner.no_route,
        }
    }
}

/// A future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct EndpointServiceFuture<F> {
    inner: Respondable<F>,
    context: ResponderContext,
    no_route: fn() -> hyper::Response,
}

impl<F, E> Future for EndpointServiceFuture<F>
where
    F: Future<Error = Result<E, hyper::Error>>,
    F::Item: IntoResponder,
    E: IntoResponder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(Some(item))) => Ok(Async::Ready(
                item.into_responder().respond(&mut self.context),
            )),
            Ok(Async::Ready(None)) => Ok(Async::Ready((self.no_route)())),
            Err(Ok(err)) => Ok(Async::Ready(
                err.into_responder().respond(&mut self.context),
            )),
            Err(Err(err)) => Err(err),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Respondable<F> {
    Polling(F),
    NoRoute,
    Done,
}

impl<F: Future> Future for Respondable<F> {
    type Item = Option<F::Item>;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::Respondable::*;
        match *self {
            Polling(ref mut t) => return t.poll().map(|s| s.map(Some)),
            NoRoute => {}
            Done => panic!(),
        }
        match mem::replace(self, Done) {
            NoRoute => Ok(Async::Ready(None)),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct EndpointServiceFactory<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    inner: Arc<EndpointServiceContext<E>>,
}

impl<E> EndpointServiceFactory<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    pub fn new(endpoint: E) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                secret_key: SecretKey::generated(),
                no_route: no_route,
            }),
        }
    }

    pub fn set_secret_key(&mut self, key: SecretKey) {
        self.inner_mut().secret_key = key;
    }

    pub fn set_no_route(&mut self, no_route: fn() -> hyper::Response) {
        self.inner_mut().no_route = no_route;
    }

    fn inner_mut(&mut self) -> &mut EndpointServiceContext<E> {
        Arc::get_mut(&mut self.inner)
            .expect("cannot get a mutable reference of inner context of EndpointServiceFactory")
    }
}

impl<E> ServiceFactory for EndpointServiceFactory<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    type Service = EndpointService<E>;

    fn new_service(&self, handle: &Handle) -> io::Result<Self::Service> {
        Ok(EndpointService {
            inner: self.inner.clone(),
            handle: handle.clone(),
        })
    }
}

pub fn no_route() -> hyper::Response {
    hyper::Response::new().with_status(StatusCode::NotFound)
}
