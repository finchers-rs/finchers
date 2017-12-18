#[macro_use]
extern crate error_chain;
extern crate finchers;

use std::string::FromUtf8Error;
use finchers::{Endpoint, EndpointError};
use finchers::endpoint::method::{get, post};
use finchers::endpoint::{body, param};
use finchers::task::BodyError;
use finchers::response::{Responder, ResponderContext, Response, ResponseBuilder, StatusCode};
use finchers::ServerBuilder;

error_chain! {
    foreign_links {
        Endpoint(EndpointError);
        Body(BodyError<FromUtf8Error>);
    }
}

impl Responder for Error {
    fn respond_to(&mut self, _: &mut ResponderContext) -> Response {
        ResponseBuilder::default()
            .status(StatusCode::NotFound)
            .finish()
    }
}

fn main() {
    // GET /foo/:id
    let endpoint1 = get(("foo", param()))
        .map(|(_, name, _)| name)
        .and_then(|(_, name): (_, String)| Ok(format!("Hello, {}", name)));

    // POST /foo/:id [String] (Content-type: text/plain; charset=utf-8)
    let endpoint2 = post(("foo", param(), body()))
        .and_then(|(name, body): (_, String, String)| Ok(format!("Hello, {} ({})", name, body)));

    let endpoint = endpoint1.or(endpoint2).with_type::<_, Error>();

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .run_http(endpoint);
}
