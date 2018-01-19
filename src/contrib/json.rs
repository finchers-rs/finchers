//!
//! JSON support (parsing/responder) based on serde_json
//!

#![allow(missing_docs)]

extern crate serde;
extern crate serde_json;

pub use self::serde_json::{Error, Value};
use self::serde::ser::Serialize;
use self::serde::de::DeserializeOwned;
use errors::StdErrorResponseBuilder;
use http::{self, header, mime, FromBody, Headers, IntoBody, IntoResponse, Request, Response};

impl FromBody for Value {
    type Error = Error;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body)
    }
}

impl IntoBody for Value {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = self.to_string();
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}

/// Represents a JSON value
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Json<T = Value>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T> ::std::ops::Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ::std::ops::DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: DeserializeOwned> FromBody for Json<T> {
    type Error = Error;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_JSON)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body).map(Json)
    }
}

impl<T: Serialize> IntoBody for Json<T> {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = serde_json::to_vec(&self.0).expect(concat!(
            "cannot serialize the value of type ",
            stringify!(T)
        ));
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}
