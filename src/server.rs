//! Definition of HTTP services for Hyper

use std::sync::Arc;

use futures::{Future, IntoFuture};
use futures::future::{AndThen, Flatten, FutureResult, Then};
use hyper;
use hyper::server::{Http, Service};

use context::{self, Context};
use endpoint::endpoint::{Endpoint, NewEndpoint};
use errors::*;
use request;
use response::Responder;


/// A wrapper for `Endpoint`s, to provide HTTP services
#[derive(Clone)]
pub struct EndpointService<E: NewEndpoint>(pub(crate) E);

impl<E: NewEndpoint> Service for EndpointService<E>
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
        let base = context::RequestInfo::new(&req, body);
        let mut ctx = Context::from(&base);

        let endpoint = self.0.new_endpoint();
        let mut result = endpoint
            .apply(&mut ctx)
            .map_err(|_| FinchersErrorKind::NotFound.into());
        if ctx.next_segment().is_some() {
            result = Err(FinchersErrorKind::NotFound.into());
        }

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
    E: NewEndpoint + Send + Sync + 'static,
    E::Item: Responder,
{
    let service = Arc::new(endpoint).into_service();
    let new_service = move || Ok(service.clone());

    let addr = addr.parse().unwrap();
    let server = Http::new().bind(&addr, new_service).unwrap();
    server.run().unwrap();
}
