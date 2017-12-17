#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use request::Request;
use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::{Task, TaskContext};
use response::{IntoResponder, Responder, ResponderContext};


/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
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
    E::Error: IntoResponder + From<EndpointError>,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<E>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let info = Request::from_hyper(req);

        let result = {
            let mut ctx = EndpointContext::new(&info);
            self.endpoint.apply(&mut ctx)
        };

        EndpointServiceFuture {
            inner: match result {
                Ok(t) => Polling(t),
                Err(e) => NotMatched(e),
            },
            ctx: Some(TaskContext { request: info }),
        }
    }
}


/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    inner: Inner<E::Task>,
    ctx: Option<TaskContext>,
}

#[allow(missing_debug_implementations)]
enum Inner<T: Task> {
    NotMatched(EndpointError),
    Polling(T),
    Done,
}
use self::Inner::*;
use std::mem;

impl<E> EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    fn poll_task(&mut self) -> Poll<E::Item, E::Error> {
        let ctx = self.ctx.as_mut().expect("cannot resolve/reject twice");
        match self.inner {
            Polling(ref mut t) => return t.poll(ctx),
            NotMatched(..) => {}
            Done => panic!(),
        }
        match mem::replace(&mut self.inner, Done) {
            NotMatched(e) => Err(e.into()),
            _ => panic!(),
        }
    }

    fn respond<T: IntoResponder>(&mut self, t: T) -> hyper::Response {
        let TaskContext { request } = self.ctx.take().expect("cannot resolve/reject twice");
        let mut ctx = ResponderContext { request: &request };
        t.into_responder().respond_to(&mut ctx).into_raw()
    }
}

impl<E> Future for EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
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
