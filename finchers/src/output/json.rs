use http::header::HeaderValue;
use http::{header, Request, Response};
use serde::Serialize;
use serde_json;
use serde_json::Value;

use super::IntoResponse;
use crate::error::{fail, Error, Never};

/// An instance of `Output` representing statically typed JSON responses.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    type Body = Vec<u8>;
    type Error = Error;

    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(fail)?;

        let mut response = Response::new(body);
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}

impl IntoResponse for Value {
    type Body = String;
    type Error = Never;

    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let body = self.to_string();
        let mut response = Response::new(body);
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}
