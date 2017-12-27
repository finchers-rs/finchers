#![allow(missing_docs)]

use std::mem;
use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use http;
use endpoint::{Endpoint, EndpointContext, NotFound};
use task::{Task, TaskContext};
use responder::{self, IntoResponder, ResponderContext};

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    endpoint: E,
    handle: Handle,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    pub(crate) fn new(endpoint: E, handle: &Handle) -> Self {
        EndpointService {
            endpoint,
            handle: handle.clone(),
        }
    }
}

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<<E::Task as Task>::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (request, body) = http::request::reconstruct(req);
        let mut cookies = http::cookie::init_cookie_jar(&request);

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
                None => Inner::NotMatched(NotFound),
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
    F::Error: IntoResponder + From<NotFound>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(Async::Ready(responder::respond(item, &mut self.context))),
            Err(err) => Ok(Async::Ready(responder::respond(err, &mut self.context))),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Inner<F> {
    Polling(F),
    NotMatched(NotFound),
    Done,
}

impl<F: Future> Future for Inner<F>
where
    F::Error: From<NotFound>,
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            Inner::Polling(ref mut t) => return t.poll(),
            Inner::NotMatched(..) => {}
            Inner::Done => panic!(),
        }
        match mem::replace(self, Inner::Done) {
            Inner::NotMatched(e) => Err(e.into()),
            _ => panic!(),
        }
    }
}
