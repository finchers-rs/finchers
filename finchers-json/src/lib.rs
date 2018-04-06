//! JSON support for finchers, powered by `serde` and `serde_json`
//!
//! Provided features are as follows:
//!
//! * `Json<T>` - represents a JSON value to be deserialized from request bodies.
//! * `JsonBody<T>` - an endpoint to parse the request body into a value of `T`.

extern crate finchers_core;
extern crate finchers_endpoint;
extern crate futures;
extern crate http;
extern crate mime;
extern crate serde;
extern crate serde_json;

use futures::{Future, Poll};
use http::header::HeaderValue;
use http::{header, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::ops::{Deref, DerefMut};
use std::{error, fmt};

use finchers_core::error::{BadRequest, HttpError};
use finchers_core::output::{Body, HttpStatus, Responder};
use finchers_core::{Bytes, Error as FinchersError, Input, Output};
use finchers_endpoint::body::FromBody;
use finchers_endpoint::{self as endpoint, Context, Endpoint};

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
        if input.media_type().map_or(true, |m| m == mime::APPLICATION_JSON) {
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

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
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
pub struct JsonOutput<T> {
    value: T,
}

impl<T: Serialize + HttpStatus> JsonOutput<T> {
    pub fn new(value: T) -> JsonOutput<T> {
        JsonOutput { value }
    }
}

impl<T: Serialize + HttpStatus> Responder for JsonOutput<T> {
    type Error = Error;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = serde_json::to_vec(&self.value).map_err(Error::Serialize)?;
        let body_len = body.len().to_string();

        let mut response = Response::new(Body::once(body));
        *response.status_mut() = self.value.status_code();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(body_len.into())
        });
        Ok(response)
    }
}

/// The error type from serde_json
#[derive(Debug)]
pub enum Error {
    /// The value of `Content-type` is invalid
    InvalidMediaType,
    /// An error during parsing the request body
    InvalidBody(serde_json::Error),
    /// during converting to HTTP response
    Serialize(serde_json::Error),
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
            Error::InvalidBody(ref e) | Error::Serialize(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            Error::InvalidMediaType => "invalid media type",
            Error::InvalidBody(ref e) | Error::Serialize(ref e) => e.description(),
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

impl HttpError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::InvalidBody(..) | Error::InvalidMediaType => StatusCode::BAD_REQUEST,
            Error::Serialize(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
