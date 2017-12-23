#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use request;
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use response::{IntoResponder, Responder, ResponderContext};
use super::server::NotFound;

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    pub fn new(endpoint: E, _handle: &Handle) -> Self {
        // TODO: clone the instance of Handle and implement it to Context
        EndpointService { endpoint }
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
    type Future = EndpointServiceFuture<E>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (request, body) = request::reconstruct(req);

        let result = {
            let mut ctx = EndpointContext::new(&request);
            self.endpoint.apply(&mut ctx)
        };

        EndpointServiceFuture {
            inner: match result {
                Some(t) => Polling(t),
                None => NotMatched,
            },
            ctx: Some(TaskContext::new(request, body)),
        }
    }
}


/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    inner: Inner<E::Task>,
    ctx: Option<TaskContext>,
}

#[allow(missing_debug_implementations)]
enum Inner<T: Task> {
    NotMatched,
    Polling(T),
    Done,
}
use self::Inner::*;
use std::mem;

impl<E> EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    fn poll_task(&mut self) -> Poll<E::Item, E::Error> {
        let ctx = self.ctx.as_mut().expect("cannot resolve/reject twice");
        match self.inner {
            Polling(ref mut t) => return t.poll(ctx),
            NotMatched => {}
            Done => panic!(),
        }
        match mem::replace(&mut self.inner, Done) {
            NotMatched => Err(NotFound.into()),
            _ => panic!(),
        }
    }

    fn respond<T: IntoResponder>(&mut self, t: T) -> hyper::Response {
        let (request, _) = self.ctx
            .take()
            .expect("cannot resolve/reject twice")
            .deconstruct();
        let mut ctx = ResponderContext { request: &request };
        t.into_responder().respond_to(&mut ctx).into_raw()
    }
}

impl<E> Future for EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.poll_task() {
            Ok(Async::Ready(item)) => Ok(Async::Ready(self.respond(item))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Ok(Async::Ready(self.respond(err))),
        }
    }
}
