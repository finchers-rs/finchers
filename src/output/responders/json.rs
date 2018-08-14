use failure::Fail;
use http::header::HeaderValue;
use http::{header, Response};
use serde::Serialize;
use serde_json;
use serde_json::Value;
use std::mem::PinMut;

use crate::error::{HttpError, Never};
use crate::input::Input;
use crate::output::payloads::Once;
use crate::output::Responder;

/// A wrapper struct representing a statically typed JSON value.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T> Responder for Json<T>
where
    T: Serialize,
{
    type Body = Once<Vec<u8>>;
    type Error = JsonSerializeError;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(|cause| JsonSerializeError { cause })?;

        let mut response = Response::new(Once::new(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}

impl Responder for Value {
    type Body = Once<String>;
    type Error = Never;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = self.to_string();

        let mut response = Response::new(Once::new(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}

/// An error type which will occur during serialize to a JSON value.
#[derive(Debug, Fail)]
#[fail(display = "failed to serialize to JSON value: {}", cause)]
pub struct JsonSerializeError {
    cause: serde_json::Error,
}

impl HttpError for JsonSerializeError {}
