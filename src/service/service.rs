#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use request;
use endpoint::{Endpoint, EndpointContext, NotFound};
use task::{Task, TaskContext};
use response::{self, IntoResponder};


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
    type Future = EndpointServiceFuture<E>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (request, body) = request::reconstruct(req);

        let mut ctx = EndpointContext::new(&request, &self.handle);
        let result = self.endpoint.apply(&mut ctx);

        let mut ctx = TaskContext::new(&request, &self.handle, body);
        let result = result.map(|t| t.launch(&mut ctx));

        EndpointServiceFuture {
            inner: match result {
                Some(fut) => Polling(fut),
                None => NotMatched,
            },
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
    inner: Inner<<E::Task as Task>::Future>,
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
        match self.inner {
            Polling(ref mut t) => return t.poll(),
            NotMatched => {}
            Done => panic!(),
        }
        match mem::replace(&mut self.inner, Done) {
            NotMatched => Err(NotFound.into()),
            _ => panic!(),
        }
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
            Ok(Async::Ready(item)) => Ok(Async::Ready(response::respond(item))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Ok(Async::Ready(response::respond(err))),
        }
    }
}
