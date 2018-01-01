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

use http::{self, CookieManager, StatusCode};
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
    cookie_manager: CookieManager,
    no_route: NoRoute,
}

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug)]
pub struct EndpointService<E> {
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
        let (request, body) = http::request::reconstruct(req);
        let mut cookies = self.inner.cookie_manager.new_cookies(request.header());

        let inner = {
            let mut ctx = EndpointContext::new(&request, &self.handle);
            match self.inner.endpoint.apply(&mut ctx) {
                Some(task) => {
                    let mut ctx = TaskContext {
                        request: &request,
                        handle: &self.handle,
                        cookies: &mut cookies,
                        body: Some(body),
                    };
                    Respondable::Polling(task.launch(&mut ctx))
                }
                None => Respondable::NoRoute(NoRoute),
            }
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
pub struct EndpointServiceFactory<E: Endpoint> {
    inner: Arc<EndpointServiceContext<E>>,
}

impl<E: Endpoint> EndpointServiceFactory<E> {
    pub fn new(endpoint: E) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                cookie_manager: CookieManager::default(),
                no_route: Default::default(),
            }),
        }
    }

    pub fn with_secret_key<K: AsRef<[u8]>>(endpoint: E, key: K) -> Self {
        EndpointServiceFactory {
            inner: Arc::new(EndpointServiceContext {
                endpoint,
                cookie_manager: CookieManager::new(key),
                no_route: Default::default(),
            }),
        }
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
