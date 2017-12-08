//!
//! JSON support (parsing/responder) based on serde_json
//!

extern crate serde;
extern crate serde_json;

pub use self::serde_json::{Error, Value};

use hyper::{header, mime};
use self::serde::ser::Serialize;
use self::serde::de::DeserializeOwned;

use endpoint::body::{body, Body};
use request::{FromBody, Request};
use response::{Responder, Response};


/// Represents a JSON value
#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: DeserializeOwned> FromBody for Json<T> {
    type Error = Error;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(false, |m| *m == mime::APPLICATION_JSON)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body).map(Json)
    }
}

impl<T: Serialize> Responder for Json<T> {
    type Error = Error;

    fn respond(self) -> Result<Response, Self::Error> {
        let body = serde_json::to_vec(&self.0)?;
        let len = body.len();
        Ok(Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(len as u64))
            .with_body(body))
    }
}

/// Create an endpoint with parsing JSON body
pub fn json_body<T: DeserializeOwned>() -> Body<Json<T>> {
    body::<Json<T>>()
}
