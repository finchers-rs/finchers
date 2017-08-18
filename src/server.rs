use std::io;
use std::fmt::Debug;

use futures::Future;
use futures::future::ok;
use hyper;
use hyper::server::Service;

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
                Box::new(f.map(|r| r.respond()).map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "handle error").into()
                }))
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