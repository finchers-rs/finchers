//! Components for parsing the JSON payload and converting to JSON values.

use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::ops::Deref;
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
    type Error = JsonError;

    fn from_data(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_err(|_| JsonError::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
        {
            serde_json::from_slice(&*body)
                .map(Json)
                .map_err(|cause| JsonError::InvalidBody { cause })
        } else {
            Err(JsonError::InvalidMediaType)
        }
    }
}

impl<T> Responder for Json<T>
where
    T: Serialize + HttpStatus,
{
    type Error = JsonError;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(|cause| JsonError::Serialize { cause })?;
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
    type Error = JsonError;

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

/// All error kinds when receiving/parsing the JSON data.
#[derive(Debug, Fail)]
pub enum JsonError {
    #[fail(display = "The value of `Content-type' is invalid")]
    InvalidMediaType,

    #[fail(display = "bad JSON data")]
    InvalidBody { cause: serde_json::Error },

    #[fail(display = "failed to serialize to JSON value")]
    Serialize { cause: serde_json::Error },
}

impl HttpError for JsonError {
    fn status_code(&self) -> StatusCode {
        match *self {
            JsonError::InvalidBody { .. } | JsonError::InvalidMediaType => StatusCode::BAD_REQUEST,
            JsonError::Serialize { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
