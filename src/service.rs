#![allow(missing_docs)]

use futures::{Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use response::Responder;
use task::Task;


/// A wrapper of a `NewEndpoint`, to provide hyper's HTTP services
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
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
    E::Error: Responder,
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
    E::Item: Responder,
    E::Error: Responder,
{
    result: Result<E::Task, Option<EndpointError>>,
    ctx: Context,
}

impl<E> Future for EndpointServiceFuture<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check the result of `Endpoint::apply()`.
        let inner = match self.result.as_mut() {
            Ok(inner) => inner,
            Err(err) => {
                let err = err.take().expect("cannot reject twice");
                return Ok(err.into_response().into());
            }
        };

        // Query the future returned from the endpoint
        let item = inner.poll(&mut self.ctx);
        // ...and convert its success/error value to `hyper::Response`.
        let item = item.map(|item| item.map(Responder::into_response))
            .map_err(Responder::into_response);

        Ok(item.unwrap_or_else(Into::into))
    }
}
