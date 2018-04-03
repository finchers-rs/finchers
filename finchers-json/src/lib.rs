//! JSON support for finchers, powered by `serde` and `serde_json`
//!
//! Provided features are as follows:
//!
//! * `Json<T>` - represents a JSON value to be deserialized from request bodies.
//! * `JsonBody<T>` - an endpoint to parse the request body into a value of `T`.

// #![doc(html_root_url = "https://docs.rs/finchers/0.10.1")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate finchers;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use finchers::futures::{Future, Poll};
use finchers::http::{header, Response, StatusCode};
use finchers::mime;

use finchers::endpoint::{self, Endpoint, EndpointContext, Outcome};
use finchers::errors::{BadRequest, Error as FinchersError, HttpError};
use finchers::request::{Bytes, FromBody, Input};
use finchers::response::{HttpStatus, Responder};

/// The error type from serde_json
#[derive(Debug)]
pub enum Error {
    /// The value of `Content-type` is invalid
    InvalidMediaType,
    /// An error during parsing the request body
    InvalidBody(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    #[inline]
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidBody(err)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidMediaType => f.write_str("The header `Content-type' must be application/json"),
            Error::InvalidBody(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            Error::InvalidMediaType => "invalid media type",
            Error::InvalidBody(ref e) => e.description(),
        }
    }

    #[inline]
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidBody(ref e) => Some(&*e),
            _ => None,
        }
    }
}

/// Represents a JSON value
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: DeserializeOwned + 'static> FromBody for Json<T> {
    type Error = BadRequest<Error>;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_or(true, |m| m == mime::APPLICATION_JSON)
        {
            serde_json::from_slice(&*body)
                .map(Json)
                .map_err(|e| BadRequest::new(Error::InvalidBody(e)))
        } else {
            Err(BadRequest::new(Error::InvalidMediaType))
        }
    }
}

/// Creates an endpoint to parse the request body to a value of `T`
pub fn json_body<T: DeserializeOwned + 'static>() -> JsonBody<T> {
    JsonBody {
        inner: endpoint::body::body(),
    }
}

#[allow(missing_docs)]
pub struct JsonBody<T> {
    inner: endpoint::body::Body<Json<T>>,
}

impl<T> Copy for JsonBody<T> {}

impl<T> Clone for JsonBody<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for JsonBody<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("JsonBody").field(&self.inner).finish()
    }
}

impl<T: DeserializeOwned + 'static> Endpoint for JsonBody<T> {
    type Item = T;
    type Future = JsonBodyFuture<T>;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        Some(JsonBodyFuture {
            inner: match self.inner.apply(input, ctx) {
                Some(inner) => inner,
                None => return None,
            },
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct JsonBodyFuture<T> {
    inner: endpoint::body::BodyFuture<Json<T>>,
}

impl<T: DeserializeOwned + 'static> Future for JsonBodyFuture<T> {
    type Item = T;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll().map(|async| async.map(|Json(body)| body))
    }
}

#[allow(missing_docs)]
pub struct JsonResponder<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for JsonResponder<T> {}

impl<T> Clone for JsonResponder<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for JsonResponder<T> {
    fn default() -> Self {
        JsonResponder {
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for JsonResponder<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonResponder").finish()
    }
}

impl<T: Serialize + HttpStatus> Responder for JsonResponder<T> {
    type Item = T;
    type Body = String;

    fn respond(&self, outcome: Outcome<T>) -> Response<String> {
        match outcome {
            Outcome::Ok(item) => json_response(&item),
            Outcome::Err(err) => json_error_response(&*err),
            Outcome::NoRoute => no_route(),
        }
    }
}

fn json_response<T: Serialize + HttpStatus>(item: &T) -> Response<String> {
    let body = serde_json::to_string(item).expect("failed to serialize a JSON body");
    make_json_response(item.status_code(), body)
}

fn json_error_response(err: &HttpError) -> Response<String> {
    make_json_response(
        err.status_code(),
        json!({
            "message": err.to_string(),
            "description": err.description(),
        }).to_string(),
    )
}

fn no_route() -> Response<String> {
    make_json_response(
        StatusCode::NOT_FOUND,
        json!({
            "message": "Not found",
        }).to_string(),
    )
}

fn make_json_response(status: StatusCode, body: String) -> Response<String> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, body.len().to_string().as_str())
        .body(body)
        .unwrap()
}
