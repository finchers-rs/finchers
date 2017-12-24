#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use request;
use endpoint::{Endpoint, EndpointContext};
use task::{Task, TaskContext};
use response::Responder;
use super::server::NotFound;

/// An HTTP service which wraps a `Endpoint`.
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
{
    pub fn new(endpoint: E, _handle: &Handle) -> Self {
        // TODO: clone the instance of Handle and implement it to Context
        EndpointService { endpoint }
    }
}

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
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

        let mut ctx = TaskContext::new(request, body);
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
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
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
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
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
    E::Item: Responder,
    E::Error: Responder + From<NotFound>,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.poll_task() {
            Ok(Async::Ready(item)) => Ok(Async::Ready(item.respond())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Ok(Async::Ready(err.respond())),
        }
    }
}
