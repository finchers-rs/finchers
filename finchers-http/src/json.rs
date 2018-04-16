//! Components for parsing the JSON payload and converting to JSON values.

use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::ops::{Deref, DerefMut};
use std::{error, fmt};
use {mime, serde_json};

use body::FromBody;
use finchers_core::error::HttpError;
use finchers_core::output::{Body, HttpStatus, Responder};
use finchers_core::{Input, Output};

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
    type Error = Error;

    fn from_body(body: Bytes, input: &mut Input) -> Result<Self, Self::Error> {
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

impl<T: Serialize + HttpStatus> Responder for Json<T> {
    type Error = Error;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(Error::Serialize)?;
        let body_len = body.len().to_string();

        let mut response = Response::new(Body::once(body));
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
