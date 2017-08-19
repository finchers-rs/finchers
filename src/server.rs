use std::sync::Arc;

use futures::{Future, Poll, Async};
use hyper::{self, StatusCode};
use hyper::server::{Http, Service};

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use errors::EndpointError;
use request;
use response::Responder;


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
        let req = request::reconstruct(req);
        let ctx = Context::new(&req);

        let endpoint = self.0.new_endpoint();
        match endpoint.apply(ctx) {
            Ok((_ctx, f)) => EndpointServiceFuture::Then(f),
            Err(err) => EndpointServiceFuture::Routing(Some(err)),
        }
    }
}

pub enum EndpointServiceFuture<F: Future<Error = StatusCode>> {
    Routing(Option<EndpointError>),
    Then(F),
}

impl<F: Future<Error = StatusCode>> Future for EndpointServiceFuture<F>
where
    F::Item: Responder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = match *self {
            EndpointServiceFuture::Routing(ref mut r) => {
                let err = r.take().expect("cannot reject twice");
                hyper::Response::new()
                    .with_status(hyper::StatusCode::NotFound)
                    .with_body(format!("{:?}", err))
            }
            EndpointServiceFuture::Then(ref mut t) => {
                match t.poll() {
                    Ok(Async::Ready(res)) => {
                        match res.respond() {
                            Ok(response) => response,
                            Err(err) => {
                                hyper::Response::new()
                                    .with_status(StatusCode::InternalServerError)
                                    .with_body(format!("{:?}", err))
                            }
                        }
                    }
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(status) => hyper::Response::new().with_status(status),
                }
            }
        };

        Ok(Async::Ready(response))
    }
}


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
