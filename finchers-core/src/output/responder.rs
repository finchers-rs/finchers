use bytes::Bytes;
use either::Either;
use error::HttpError;
use http::header::HeaderValue;
use http::{header, Response};
use input::Input;
use never::Never;
use std::fmt;

use super::body::Body;

pub type Output = Response<Body>;

const TEXT_PLAIN: &str = "text/plain; charset=utf-8";
const OCTET_STREAM: &str = "application/octet-stream";

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
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(self.map(Into::into))
    }
}

impl Responder for Bytes {
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let mut response = Response::new(Body::once(self));
        content_type(&mut response, OCTET_STREAM);
        content_length(&mut response);

        Ok(response)
    }
}

impl Responder for String {
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let mut response = Response::new(Body::once(self));
        content_type(&mut response, TEXT_PLAIN);
        content_length(&mut response);

        Ok(response)
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: Responder,
{
    type Error = Either<T::Error, E::Error>;

    fn respond(self, input: &Input) -> Result<Output, Self::Error> {
        match self {
            Ok(ok) => ok.respond(input).map_err(Either::Left),
            Err(e) => e.respond(input).map_err(Either::Right),
        }
    }
}

impl<L, R> Responder for Either<L, R>
where
    L: Responder,
    R: Responder,
{
    type Error = Either<L::Error, R::Error>;

    fn respond(self, input: &Input) -> Result<Output, Self::Error> {
        match self {
            Either::Left(l) => l.respond(input).map_err(Either::Left),
            Either::Right(r) => r.respond(input).map_err(Either::Right),
        }
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
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };

        let mut response = Response::new(Body::once(body));
        content_type(&mut response, TEXT_PLAIN);
        content_length(&mut response);

        Ok(response)
    }
}

fn content_type<T>(response: &mut Response<T>, value: &'static str) {
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(value));
}

fn content_length(response: &mut Response<Body>) {
    if let Some(body_len) = response.body().len() {
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(Bytes::from(body_len.to_string()))
        });
    }
}
