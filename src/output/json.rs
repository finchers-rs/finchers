use http::header::HeaderValue;
use http::{header, Response};
use serde::Serialize;
use serde_json;
use serde_json::Value;

use super::payload::Once;
use super::{Output, OutputContext};
use crate::error::{fail, Error, Never};

/// An instance of `Responder` representing statically typed JSON responses.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T: Serialize> Output for Json<T> {
    type Body = Once<Vec<u8>>;
    type Error = Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let body = if cx.is_pretty() {
            serde_json::to_vec_pretty(&self.0).map_err(fail)?
        } else {
            serde_json::to_vec(&self.0).map_err(fail)?
        };

        let mut response = Response::new(Once::new(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}

impl Output for Value {
    type Body = Once<String>;
    type Error = Never;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let body = if cx.is_pretty() {
            format!("{:#}", self)
        } else {
            format!("{}", self)
        };

        let mut response = Response::new(Once::new(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(response)
    }
}
