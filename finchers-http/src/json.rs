//! Components for parsing the JSON payload and converting to JSON values.

use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::ops::Deref;
use std::{error, fmt};
use {mime, serde_json};

use body::FromData;
use finchers_core::error::HttpError;
use finchers_core::output::{HttpStatus, Responder, ResponseBody};
use finchers_core::{Input, Output};

/// A wrapper struct representing a statically typed JSON value.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Consume itself and return the instance of inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

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

impl<T> FromData for Json<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;

    fn from_data(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_err(|_| Error::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
        {
            serde_json::from_slice(&*body).map(Json).map_err(Error::InvalidBody)
        } else {
            Err(Error::InvalidMediaType)
        }
    }
}

impl<T> Responder for Json<T>
where
    T: Serialize + HttpStatus,
{
    type Error = Error;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(Error::Serialize)?;
        let body_len = body.len().to_string();

        let mut response = Response::new(ResponseBody::once(body));
        *response.status_mut() = self.0.status_code();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(body_len.into())
        });
        Ok(response)
    }
}

/// A type representing a JSON value.
///
/// This type is used as an output value of the endpoint or error handler.
#[derive(Debug)]
pub struct JsonValue {
    value: serde_json::Value,
    status: StatusCode,
}

impl From<serde_json::Value> for JsonValue {
    fn from(value: serde_json::Value) -> Self {
        JsonValue::new(value, StatusCode::OK)
    }
}

impl JsonValue {
    pub fn new(value: serde_json::Value, status: StatusCode) -> JsonValue {
        JsonValue { value, status }
    }
}

impl Responder for JsonValue {
    type Error = Error;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = self.value.to_string();
        let body_len = body.len().to_string();

        let mut response = Response::new(ResponseBody::once(body));
        *response.status_mut() = self.status;
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
