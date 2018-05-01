use std::borrow::Cow;
use std::fmt;

use bytes::Bytes;
use either::Either;
use failure::Error;
use http::header::HeaderValue;
use http::{header, Response};

use super::body::ResponseBody;
use input::Input;
use never::Never;

pub type Output = Response<ResponseBody>;

const TEXT_PLAIN: &str = "text/plain; charset=utf-8";

/// Trait representing the conversion to an HTTP response.
pub trait Responder {
    /// The error type returned from "respond".
    type Error: Into<Error>;

    /// Create an HTTP response from the value of "Self".
    fn respond(self, input: &Input) -> Result<Output, Self::Error>;
}

impl<T> Responder for Response<T>
where
    T: Into<ResponseBody>,
{
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(self.map(Into::into))
    }
}

impl Responder for &'static str {
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(make_response(Bytes::from_static(self.as_bytes()), TEXT_PLAIN))
    }
}

impl Responder for String {
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(make_response(Bytes::from(self), TEXT_PLAIN))
    }
}

impl Responder for Cow<'static, str> {
    type Error = Never;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = match self {
            Cow::Borrowed(s) => Bytes::from_static(s.as_bytes()),
            Cow::Owned(s) => Bytes::from(s),
        };
        Ok(make_response(body, TEXT_PLAIN))
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: Into<Error>,
{
    type Error = Error;

    fn respond(self, input: &Input) -> Result<Output, Self::Error> {
        self.map_err(Into::<Error>::into)?.respond(input).map_err(Into::into)
    }
}

impl<L, R> Responder for Either<L, R>
where
    L: Responder,
    R: Responder,
{
    type Error = Error;

    fn respond(self, input: &Input) -> Result<Output, Self::Error> {
        match self {
            Either::Left(l) => l.respond(input).map_err(Into::into),
            Either::Right(r) => r.respond(input).map_err(Into::into),
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

        let mut response = Response::new(ResponseBody::once(body));
        content_type(&mut response, TEXT_PLAIN);
        content_length(&mut response);

        Ok(response)
    }
}

fn make_response(body: Bytes, m: &'static str) -> Output {
    let mut response = Response::new(ResponseBody::once(body));
    content_type(&mut response, m);
    content_length(&mut response);
    response
}

fn content_type<T>(response: &mut Response<T>, value: &'static str) {
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(value));
}

fn content_length(response: &mut Response<ResponseBody>) {
    if let Some(body_len) = response.body().len() {
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(Bytes::from(body_len.to_string()))
        });
    }
}
