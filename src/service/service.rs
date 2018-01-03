#![allow(missing_docs)]

use std::io;
use std::fmt;
use std::error::Error;
use std::mem;
use std::sync::Arc;

use futures::{Async, Future, Poll};
use hyper;
use hyper::server::Service;
use tokio_core::reactor::Handle;

use http::{self, Cookies, SecretKey, StatusCode};
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use responder::{ErrorResponder, IntoResponder};
use responder::inner::{respond, ResponderContext};
use super::ServiceFactory;

/// An error represents which represents that
/// the matched route was not found.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("not found")
    }
}

impl Error for NoRoute {
    fn description(&self) -> &str {
        "not found"
    }
}

impl ErrorResponder for NoRoute {
    fn status(&self) -> StatusCode {
        StatusCode::NotFound
    }

    fn message(&self) -> Option<String> {
        None
    }
}

/// The inner representation of `EndpointService`.
#[derive(Debug)]
struct EndpointServiceContext<E> {
    endpoint: E,
    secret_key: SecretKey,
    no_route: NoRoute,
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
                Respondable::Polling(task.launch(&mut ctx))
            }
            None => Respondable::NoRoute(NoRoute),
        };

        EndpointServiceFuture {
            inner,
            context: ResponderContext { request, cookies },
        }
    }
}

/// A future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct EndpointServiceFuture<F> {
    inner: Respondable<F>,
    context: ResponderContext,
}

impl<F> Future for EndpointServiceFuture<F>
where
    F: Future,
    F::Item: IntoResponder,
    F::Error: IntoResponder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(Async::Ready(respond(item, &mut self.context))),
            Err(Ok(err)) => Ok(Async::Ready(respond(err, &mut self.context))),
            Err(Err(no_route)) => Ok(Async::Ready(respond(no_route, &mut self.context))),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Respondable<F> {
    Polling(F),
    NoRoute(NoRoute),
    Done,
}

impl<F: Future> Future for Respondable<F> {
    type Item = F::Item;
    type Error = Result<F::Error, NoRoute>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::Respondable::*;
        match *self {
            Polling(ref mut t) => return t.poll().map_err(Ok),
            NoRoute(..) => {}
            Done => panic!(),
        }
        match mem::replace(self, Done) {
            NoRoute(e) => Err(Err(e)),
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
        Self::with_secret_key(endpoint, SecretKey::generated())
    }

    pub fn with_secret_key(endpoint: E, secret_key: SecretKey) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                secret_key,
                no_route: Default::default(),
            }),
        }
    }

    pub fn set_secret_key(&mut self, key: SecretKey) {
        let inner = Arc::get_mut(&mut self.inner)
            .expect("cannot get a mutable reference of inner context of EndpointServiceFactory");
        inner.secret_key = key;
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
