#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use response::{IntoResponder, Responder};
use task::Task;


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
        let mut ctx = Context::from_hyper(req);
        let result = self.endpoint.apply(&mut ctx);
        EndpointServiceFuture {
            result: result.map_err(Some),
            ctx,
        }
    }
}


/// A future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    result: Result<E::Task, Option<EndpointError>>,
    ctx: Context,
}

impl<E> EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    fn poll_task(&mut self) -> Poll<E::Item, E::Error> {
        match self.result {
            Ok(ref mut inner) => inner.poll(&mut self.ctx),
            Err(ref mut err) => {
                let err = err.take().expect("cannot reject twice");
                Err(err.into())
            }
        }
    }

    fn respond<T: IntoResponder>(&mut self, t: T) -> hyper::Response {
        t.into_responder().respond_to(&mut self.ctx).into_raw()
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
