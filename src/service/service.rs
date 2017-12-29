#![allow(missing_docs)]

use std::mem;
use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use http::{self, CookieManager};
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use responder::IntoResponder;
use responder::inner::{respond, ResponderContext};
use super::NoRoute;

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    pub(crate) endpoint: E,
    pub(crate) handle: Handle,
    pub(crate) cookie_manager: CookieManager,
    pub(crate) no_route: NoRoute,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    pub fn cookie_manager(&mut self) -> &mut CookieManager {
        &mut self.cookie_manager
    }
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
        let mut cookies = self.cookie_manager.new_cookies(request.header());

        let inner = {
            let mut ctx = EndpointContext::new(&request, &self.handle);
            match self.endpoint.apply(&mut ctx) {
                Some(task) => {
                    let mut ctx = TaskContext {
                        request: &request,
                        handle: &self.handle,
                        cookies: &mut cookies,
                        body: Some(body),
                    };
                    Inner::Polling(task.launch(&mut ctx))
                }
                None => Inner::NoRoute(NoRoute),
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
    inner: Inner<F>,
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
pub(crate) enum Inner<F> {
    Polling(F),
    NoRoute(NoRoute),
    Done,
}

impl<F: Future> Future for Inner<F> {
    type Item = F::Item;
    type Error = Result<F::Error, NoRoute>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            Inner::Polling(ref mut t) => return t.poll().map_err(Ok),
            Inner::NoRoute(..) => {}
            Inner::Done => panic!(),
        }
        match mem::replace(self, Inner::Done) {
            Inner::NoRoute(e) => Err(Err(e)),
            _ => panic!(),
        }
    }
}
