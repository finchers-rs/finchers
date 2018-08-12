//! Components for parsing the JSON payload and converting to JSON values.

use std::mem::PinMut;
use std::ops::Deref;

use bytes::Bytes;
use failure::Fail;
use http::header::{HeaderMap, HeaderValue};
use http::{header, Response, StatusCode};
use mime;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json;

use either::Either;
use error::{HttpError, Never};
use input::{FromBody, Input};
use output::payloads::Once;
use output::Responder;

/// Trait representing additional information for constructing an HTTP response.
///
/// This trait is used as a helper to define the implementation of `Responder`.
pub trait HttpResponse {
    /// Returns a HTTP status code.
    fn status_code(&self) -> StatusCode;

    /// Append header values to given header map.
    #[allow(unused_variables)]
    fn append_headers(&self, headers: &mut HeaderMap<HeaderValue>) {}
}

macro_rules! impl_status {
    ($($t:ty),*) => {$(
        impl HttpResponse for $t {
            fn status_code(&self) -> StatusCode {
                StatusCode::OK
            }
        }
    )*};
}

impl_status!(bool, char, f32, f64, String, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl<L, R> HttpResponse for Either<L, R>
where
    L: HttpResponse,
    R: HttpResponse,
{
    fn status_code(&self) -> StatusCode {
        match *self {
            Either::Left(ref l) => l.status_code(),
            Either::Right(ref r) => r.status_code(),
        }
    }

    fn append_headers(&self, headers: &mut HeaderMap<HeaderValue>) {
        match *self {
            Either::Left(ref l) => l.append_headers(headers),
            Either::Right(ref r) => r.append_headers(headers),
        }
    }
}

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

impl<T> FromBody for Json<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = JsonParseError;

    fn from_body(body: Bytes, input: PinMut<Input>) -> Result<Self, Self::Error> {
        if input
            .content_type()
            .map_err(|_| JsonParseError::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
        {
            serde_json::from_slice(&*body)
                .map(Json)
                .map_err(|cause| JsonParseError::Parse { cause })
        } else {
            Err(JsonParseError::InvalidMediaType)
        }
    }
}

impl<T> Responder for Json<T>
where
    T: Serialize + HttpResponse,
{
    type Body = Once<Vec<u8>>;
    type Error = JsonSerializeError;

    fn respond(self, _: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(|cause| JsonSerializeError { cause })?;

        let mut response = Response::new(Once::new(body));
        *response.status_mut() = self.0.status_code();
        self.0.append_headers(response.headers_mut());
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

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
    #[allow(missing_docs)]
    pub fn new(value: serde_json::Value, status: StatusCode) -> JsonValue {
        JsonValue { value, status }
    }
}

impl Responder for JsonValue {
    type Body = Once<String>;
    type Error = Never;

    fn respond(self, _: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = self.value.to_string();

        let mut response = Response::new(Once::new(body));
        *response.status_mut() = self.status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}

/// An error type which will occur during transforming a payload to a JSON value.
#[derive(Debug, Fail)]
pub enum JsonParseError {
    #[allow(missing_docs)]
    #[fail(display = "The value of `Content-type' is invalid")]
    InvalidMediaType,

    #[allow(missing_docs)]
    #[fail(display = "Failed to parse the payload to a JSON value")]
    Parse { cause: serde_json::Error },
}

impl HttpError for JsonParseError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// An error type which will occur during serialize to a JSON value.
#[derive(Debug, Fail)]
#[fail(display = "failed to serialize to JSON value: {}", cause)]
pub struct JsonSerializeError {
    cause: serde_json::Error,
}

impl Deref for JsonSerializeError {
    type Target = serde_json::Error;

    fn deref(&self) -> &Self::Target {
        &self.cause
    }
}

impl HttpError for JsonSerializeError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
