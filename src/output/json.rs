use http::header::HeaderValue;
use http::{header, Response};
use serde::Serialize;
use serde_json;
use serde_json::Value;
use std::mem::PinMut;

use crate::error::{fail, Error, Never};
use crate::input::Input;
use crate::output::payload::Once;
use crate::output::Responder;

/// An instance of `Responder` representing statically typed JSON responses.
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
    type Error = Error;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = serde_json::to_vec(&self.0).map_err(fail)?;

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
