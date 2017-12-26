#![cfg_attr(rustfmt, rustfmt_skip)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;

use std::sync::Arc;
use std::string::ParseError;
use std::error::Error as StdError;

use finchers::{Endpoint, NotFound, Responder};
use finchers::endpoint::method::{get, post};
use finchers::endpoint::{body, path};
use finchers::http::{self, StatusCode, StringBodyError};
use finchers::service::ServerBuilder;


error_chain! {
    types { Error, ErrorKind, ResultExt, Result; }
    foreign_links {
        NotFound(NotFound);
        ParsePath(ParseError);
        BodyRecv(http::Error);
        StringBody(StringBodyError);
    }
}

impl Responder for Error {
    type Body = String;

    fn status(&self) -> StatusCode {
        match *self.kind() {
            ErrorKind::NotFound(..) => StatusCode::NotFound,
            ErrorKind::BodyRecv(..) => StatusCode::InternalServerError,
            _ => StatusCode::BadRequest,
        }
    }

    fn body(&mut self) -> Option<Self::Body> {
        Some(format!("{}: {}", self.description(), self.to_string()))
    }
}

fn main() {
    // GET /hello/:id
    let endpoint1 = get(("hello" , path()))
        .and_then(|(_, name): (_, String)| -> Result<_> {
            Ok(format!("Hello, {}", name))
        });

    // POST /foo [String] (Content-type: text/plain; charset=utf-8)
    let endpoint2 = post(("hello", body()))
        .and_then(|(_, body): (_, String)| {
            Ok(format!("Received: {}", body))
        });

    let endpoint = choice!(endpoint1, endpoint2);

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .serve(Arc::new(endpoint));
}
