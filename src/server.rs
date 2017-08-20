//! Definition of HTTP services for Hyper

use std::sync::Arc;

use futures::{Future, Poll, Async};
use hyper;
use hyper::server::{Http, Service};

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use errors::*;
use request;
use response::Responder;


/// A wrapper for `NewEndpoint`s, to provide HTTP services
pub struct EndpointService<E: NewEndpoint>(pub(crate) E);

impl<E: NewEndpoint> Service for EndpointService<E>
where
    E::Item: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointServiceFuture<E::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (req, body) = request::reconstruct(req);
        let ctx = Context::new(&req);

        let endpoint = self.0.new_endpoint();
        match endpoint.apply(ctx, Some(body)) {
            Ok((_ctx, _body, f)) => EndpointServiceFuture::Then(f),
            Err((err, _body)) => EndpointServiceFuture::Routing(Some(err)),
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


/// Start the HTTP server, with given endpoint factory and listener address.
pub fn run_http<E: NewEndpoint + Send + Sync + 'static>(new_endpoint: E, addr: &str)
where
    E::Item: Responder,
{
    let new_endpoint = Arc::new(new_endpoint);

    let addr = addr.parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || Ok(new_endpoint.clone().into_service()))
        .unwrap();
    server.run().unwrap();

}
