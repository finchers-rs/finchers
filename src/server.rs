use std::io;
use std::fmt::Debug;
use std::sync::Arc;
use std::error;

use futures::Future;
use futures::future::ok;
use hyper;
use hyper::server::{Http, Service};

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use request::Request;
use response::Responder;


pub struct EndpointService<E: NewEndpoint>(pub E);

impl<E: NewEndpoint> Service for EndpointService<E>
where
    E::Item: Responder,
    E::Error: Debug,
    E::Future: 'static,
    <E::Item as Responder>::Error: error::Error + Send + Sync + 'static
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (method, uri, _version, headers, body) = req.deconstruct();
        let req = Request {
            method,
            uri,
            headers,
            body: Some(body),
        };
        let ctx = Context::new(&req);
        let endpoint = self.0.new_endpoint();
        match endpoint.apply(ctx) {
            Ok((_ctx, f)) => {
                Box::new(
                    f.map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "handle error").into()
                    }).and_then(|r| {
                            r.respond().map_err(
                                |err| io::Error::new(io::ErrorKind::Other, err).into(),
                            )
                        }),
                )
            }
            Err(err) => {
                Box::new(ok(
                    hyper::Response::new()
                        .with_status(hyper::StatusCode::NotFound)
                        .with_body(format!("{:?}", err)),
                ))
            }
        }
    }
}


pub fn run_http<E: NewEndpoint + Send + Sync + 'static>(new_endpoint: E, addr: &str)
where
    E::Item: Responder,
    E::Error: Debug,
    <E::Item as Responder>::Error: error::Error + Send + Sync,
{
    let new_endpoint = Arc::new(new_endpoint);

    let addr = addr.parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || Ok(EndpointService(new_endpoint.clone())))
        .unwrap();
    server.run().unwrap();

}
