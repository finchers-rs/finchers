#[macro_use]
extern crate error_chain;
extern crate finchers;

use std::string::{FromUtf8Error, ParseError};
use finchers::Endpoint;
use finchers::endpoint::method::{get, post};
use finchers::endpoint::{body, path};
use finchers::request::BodyError;
use finchers::response::{Responder, ResponderContext, Response, ResponseBuilder, StatusCode};
use finchers::ServerBuilder;
use finchers::server::NotFound;

error_chain! {
    foreign_links {
        NotFound(NotFound);
        ParsePath(ParseError);
        BodyRecv(BodyError);
        FromUtf8(FromUtf8Error);
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
    let endpoint1 = get(("foo", path())).and_then(|(_, name): (_, String)| Ok(format!("Hello, {}", name)));

    // POST /foo/:id [String] (Content-type: text/plain; charset=utf-8)
    let endpoint2 = post(("foo", path(), body()))
        .and_then(|(_, name, body): (_, String, String)| Ok(format!("Hello, {} ({})", name, body)));

    let endpoint = endpoint1.or(endpoint2).with_type::<_, Error>();

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .run_http(endpoint);
}
