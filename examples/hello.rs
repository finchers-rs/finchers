#[macro_use]
extern crate error_chain;
extern crate finchers;
#[macro_use]
extern crate serde_json;

use std::string::{FromUtf8Error, ParseError};
use std::error::Error as StdError;
use serde_json::Value;

use finchers::Endpoint;
use finchers::endpoint::method::{get, post};
use finchers::endpoint::{body, path};
use finchers::request::BodyError;
use finchers::response::{Responder, StatusCode};
use finchers::ServerBuilder;
use finchers::server::NotFound;


error_chain! {
    types { MyError, MyErrorKind, ResultExt; }
    foreign_links {
        NotFound(NotFound);
        ParsePath(ParseError);
        BodyRecv(BodyError);
        FromUtf8(FromUtf8Error);
    }
}

impl Responder for MyError {
    type Body = Value;

    fn status(&self) -> StatusCode {
        match *self.kind() {
            MyErrorKind::NotFound(..) => StatusCode::NotFound,
            _ => StatusCode::BadRequest,
        }
    }

    fn body(&mut self) -> Option<Value> {
        Some(json!({
            "error_code": self.status().to_string(),
            "description": self.description(),
            "message": self.to_string(),
        }))
    }
}

fn main() {
    // GET /foo/:id
    let endpoint1 =
        get(("foo", path())).and_then(|(_, name): (_, String)| Ok(format!("Hello, {}", name)) as Result<_, MyError>);

    // POST /foo/:id [String] (Content-type: text/plain; charset=utf-8)
    let endpoint2 = post(("foo", path(), body()))
        .and_then(|(_, name, body): (_, String, String)| Ok(format!("Hello, {} ({})", name, body)));

    let endpoint = endpoint1.or(endpoint2);

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .run_http(endpoint);
}
