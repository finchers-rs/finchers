#![allow(missing_docs)]

use futures::{Async, Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use response::{IntoResponder, Responder, Response, ResponseBuilder, StatusCode};
use task::Task;


/// A wrapper of a `NewEndpoint`, to provide hyper's HTTP services
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
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
    E::Error: IntoResponder,
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


/// The returned future from `EndpointService::call()`
#[derive(Debug)]
pub struct EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    result: Result<E::Task, Option<EndpointError>>,
    ctx: Context,
}

impl<E> Future for EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = match self.result {
            Ok(ref mut inner) => match inner.poll(&mut self.ctx) {
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Ok(Async::Ready(item)) => item.into_responder().respond_to(&mut self.ctx),
                Err(err) => err.into_responder().respond_to(&mut self.ctx),
            },
            Err(ref mut err) => {
                let err = err.take().expect("cannot reject twice");
                err.into_responder().respond_to(&mut self.ctx)
            }
        };
        Ok(Async::Ready(response.into_raw()))
    }
}


#[derive(Debug)]
pub struct EndpointErrorResponder(EndpointError);

impl Responder for EndpointErrorResponder {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        ResponseBuilder::default()
            .status(StatusCode::NotFound)
            .finish()
    }
}

impl IntoResponder for EndpointError {
    type Responder = EndpointErrorResponder;
    fn into_responder(self) -> EndpointErrorResponder {
        EndpointErrorResponder(self)
    }
}
