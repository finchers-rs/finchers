//! Definition of HTTP services for Hyper

use std::cell::RefCell;
use std::sync::Arc;

use futures::{Future, IntoFuture};
use futures::future::{AndThen, Flatten, FutureResult, Then};
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
    type Future = Then<
        AndThen<
            Flatten<FutureResult<E::Future, FinchersError>>,
            FinchersResult<hyper::Response>,
            fn(E::Item) -> FinchersResult<hyper::Response>,
        >,
        Result<hyper::Response, hyper::Error>,
        fn(FinchersResult<hyper::Response>)
            -> Result<hyper::Response, hyper::Error>,
    >;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (req, body) = request::reconstruct(req);
        let body = RefCell::new(Some(body));
        let ctx = Context::new(&req, &body);

        let (_ctx, result) = self.0.apply(ctx);

        result
            .into_future()
            .flatten()
            .and_then(
                (|res| {
                    res.respond()
                        .map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
                }) as fn(E::Item) -> FinchersResult<hyper::Response>,
            )
            .then(|response| {
                Ok(response.unwrap_or_else(|err| err.into_response()))
            })
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
