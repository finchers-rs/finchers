//!
//! JSON support (parsing/responder) based on serde_json
//!

#![allow(missing_docs)]

extern crate serde;
extern crate serde_json;

pub use self::serde_json::{Error, Value};
use self::serde::ser::Serialize;
use self::serde::de::DeserializeOwned;

use std::fmt;
use std::error;
use endpoint::body::{body, Body};
use http::{self, header, mime, FromBody, Headers, IntoBody, Request, StatusCode};
use responder::Responder;

impl FromBody for Value {
    type Error = JsonError;

    fn validate(req: &Request) -> Result<(), Self::Error> {
        if req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
        {
            Ok(())
        } else {
            Err(JsonError::BadRequest)
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body).map_err(JsonError::Parsing)
    }
}

impl IntoBody for Value {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = self.to_string();
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}

/// Represents a JSON value
#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: DeserializeOwned> FromBody for Json<T> {
    type Error = JsonError;

    fn validate(req: &Request) -> Result<(), Self::Error> {
        if req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
        {
            Ok(())
        } else {
            Err(JsonError::BadRequest)
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body)
            .map(Json)
            .map_err(JsonError::Parsing)
    }
}

impl<T: Serialize> IntoBody for Json<T> {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = serde_json::to_vec(&self.0).expect(concat!(
            "cannot serialize the value of type ",
            stringify!(T)
        ));
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}

/// Create an endpoint with parsing JSON body
pub fn json_body<T: DeserializeOwned, E>() -> Body<Json<T>, E>
where
    E: From<http::Error> + From<JsonError>,
{
    body::<Json<T>, E>()
}

#[derive(Debug)]
pub enum JsonError {
    BadRequest,
    Parsing(Error),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsonError::BadRequest => f.write_str("bad request"),
            JsonError::Parsing(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for JsonError {
    fn description(&self) -> &str {
        match *self {
            JsonError::BadRequest => "bad request",
            JsonError::Parsing(ref e) => error::Error::description(e),
        }
    }
}

impl Responder for JsonError {
    type Body = String;

    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
    }

    fn body(&mut self) -> Option<Self::Body> {
        Some(format!("{}: {}", error::Error::description(self), self))
    }
}
