use std::borrow::Cow;
use std::fmt;

use http::header::HeaderValue;
use http::{header, Response};
use hyper::body::Payload;

use super::body::{once, Once};
use crate::either::Either;
use crate::error::{Error, Never};
use crate::input::Input;

/// Trait representing the conversion to an HTTP response.
#[allow(missing_docs)]
pub trait Responder {
    type Body: Payload;
    type Error: Into<Error>;

    /// Consume `self` and construct a new HTTP response.
    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error>;
}

impl<T: Payload> Responder for Response<T> {
    type Body = T;
    type Error = Never;

    #[inline(always)]
    fn respond(self, _: &Input) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self)
    }
}

#[derive(Debug)]
pub struct Text<T>(T);

impl<T: AsRef<str>> AsRef<[u8]> for Text<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

impl<T: AsRef<str> + Send + 'static> Responder for Text<T> {
    type Body = Once<Self>;
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(once(self));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        Ok(response)
    }
}

impl Responder for &'static str {
    type Body = Once<Text<Self>>;
    type Error = Never;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(input)
    }
}

impl Responder for String {
    type Body = Once<Text<Self>>;
    type Error = Never;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(input)
    }
}

impl Responder for Cow<'static, str> {
    type Body = Once<Text<Self>>;
    type Error = Never;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(input)
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    Error: From<E>,
{
    type Body = T::Body;
    type Error = Error;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        self?.respond(input).map_err(Into::into)
    }
}

impl<L, R> Responder for Either<L, R>
where
    L: Responder,
    R: Responder,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        match self {
            Either::Left(l) => l
                .respond(input)
                .map(|res| res.map(Either::Left))
                .map_err(Into::into),
            Either::Right(r) => r
                .respond(input)
                .map(|res| res.map(Either::Right))
                .map_err(Into::into),
        }
    }
}

/// A helper struct for creating the response from types which implements `fmt::Debug`.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T> {
    value: T,
    pretty: bool,
}

impl<T: fmt::Debug> Debug<T> {
    /// Create an instance of `Debug` from an value whose type has an implementation of
    /// `fmt::Debug`.
    pub fn new(value: T) -> Debug<T> {
        Debug {
            value,
            pretty: false,
        }
    }

    /// Set whether this responder uses the pretty-printed specifier (`"{:#?}"`) or not.
    pub fn pretty(self, enabled: bool) -> Self {
        Debug {
            pretty: enabled,
            ..self
        }
    }
}

impl<T: fmt::Debug> Responder for Debug<T> {
    type Body = Once<Text<String>>;
    type Error = Never;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
        let body = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };
        Text(body).respond(input)
    }
}
