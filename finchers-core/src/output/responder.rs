use bytes::Bytes;
use error::HttpError;
use http::header::HeaderValue;
use http::{header, Response};
use input::Input;
use std::fmt;

use super::body::Body;

pub type Output = Response<Body>;

/// Trait representing the conversion to an HTTP response.
pub trait Responder {
    /// The error type returned from "respond".
    type Error: HttpError;

    /// Create an HTTP response from the value of "Self".
    fn respond(self, input: &Input) -> Result<Output, Self::Error>;
}

impl<T> Responder for Response<T>
where
    T: Into<Body>,
{
    type Error = !;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(self.map(Into::into))
    }
}

/// A helper struct for creating the response from types which implements `fmt::Debug`.
///
/// This wrapper is only for debugging and should not use in the production code.
pub struct Debug {
    value: Box<fmt::Debug + Send + 'static>,
    pretty: bool,
}

impl Debug {
    /// Create an instance of "Debug" from an value
    /// whose type has an implementation of "fmt::Debug".
    pub fn new<T>(value: T) -> Debug
    where
        T: fmt::Debug + Send + 'static,
    {
        Debug {
            value: Box::new(value),
            pretty: false,
        }
    }

    /// Set whether this responder uses the pretty-printed specifier (":?") or not.
    pub fn pretty(mut self, enabled: bool) -> Self {
        self.pretty = enabled;
        self
    }
}

impl Responder for Debug {
    type Error = !;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };
        let body_len = body.len().to_string();

        let mut response = Response::new(Body::once(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(Bytes::from(body_len))
        });

        Ok(response)
    }
}
