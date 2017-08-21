//! Definition of HTTP services for Hyper

use std::cell::RefCell;
use std::sync::Arc;

use futures::{Future, Poll, Async};
use hyper;
use hyper::server::{Http, Service};

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request;
use response::Responder;


/// A wrapper for `Endpoint`s, to provide HTTP services
pub struct EndpointService<E: Endpoint>(pub(crate) E);

impl<E: Endpoint + Clone> Clone for EndpointService<E> {
    fn clone(&self) -> Self {
        Self { 0: self.0.clone() }
    }
}

impl<E: Endpoint> Service for EndpointService<E>
where
    E::Item: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<E::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (req, body) = request::reconstruct(req);
        let body = RefCell::new(Some(body));
        let ctx = Context::new(&req, &body);

        match self.0.apply(ctx) {
            (_ctx, Ok(f)) => EndpointServiceFuture::Then(f),
            (_ctx, Err(err)) => EndpointServiceFuture::Routing(Some(err)),
        }
    }
}

#[allow(missing_docs)]
pub enum EndpointServiceFuture<F: Future<Error = FinchersError>> {
    Routing(Option<FinchersError>),
    Then(F),
}

impl<F: Future<Error = FinchersError>> Future for EndpointServiceFuture<F>
where
    F::Item: Responder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = match *self {
            EndpointServiceFuture::Routing(ref mut r) => {
                let err = r.take().expect("cannot reject twice");
                Err(err)
            }
            EndpointServiceFuture::Then(ref mut t) => {
                match t.poll() {
                    Ok(Async::Ready(res)) => {
                        match res.respond() {
                            Ok(response) => Ok(response),
                            Err(err) => Err(FinchersErrorKind::Responder(Box::new(err)).into()),
                        }
                    }
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => Err(err),
                }
            }
        };

        Ok(Async::Ready(response.unwrap_or_else(|err| {
            hyper::Response::new().with_status(err.into_status())
        })))
    }
}


/// Start the HTTP server, with given endpoint and listener address.
pub fn run_http<E>(endpoint: E, addr: &str)
where
    E: Endpoint + Send + Sync + 'static,
    E::Item: Responder,
{
    let service = Arc::new(endpoint).into_service();
    let new_service = move || Ok(service.clone());

    let addr = addr.parse().unwrap();
    let server = Http::new().bind(&addr, new_service).unwrap();
    server.run().unwrap();
}
