use std::borrow::Cow;
use std::fmt;

use bytes::Bytes;
use either::Either;
use http::header::HeaderValue;
use http::{header, Response};

use super::body::ResponseBody;
use error::Error;
use input::Input;

const TEXT_PLAIN: &str = "text/plain; charset=utf-8";

/// A type alias of value which will be returned from `Responder`.
pub type Output = Response<ResponseBody>;

/// Trait representing the conversion to an HTTP response.
pub trait Responder {
    /// Consume `self` and construct a new HTTP response.
    fn respond(self, input: &Input) -> Result<Output, Error>;
}

impl<T> Responder for Response<T>
where
    T: Into<ResponseBody>,
{
    fn respond(self, _: &Input) -> Result<Output, Error> {
        Ok(self.map(Into::into))
    }
}

impl Responder for &'static str {
    fn respond(self, _: &Input) -> Result<Output, Error> {
        Ok(make_response(
            Bytes::from_static(self.as_bytes()),
            TEXT_PLAIN,
        ))
    }
}

impl Responder for String {
    fn respond(self, _: &Input) -> Result<Output, Error> {
        Ok(make_response(Bytes::from(self), TEXT_PLAIN))
    }
}

impl Responder for Cow<'static, str> {
    fn respond(self, _: &Input) -> Result<Output, Error> {
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
    fn respond(self, input: &Input) -> Result<Output, Error> {
        self.map_err(Into::<Error>::into)?
            .respond(input)
            .map_err(Into::into)
    }
}

impl<L, R> Responder for Either<L, R>
where
    L: Responder,
    R: Responder,
{
    fn respond(self, input: &Input) -> Result<Output, Error> {
        match self {
            Either::Left(l) => l.respond(input),
            Either::Right(r) => r.respond(input),
        }
    }
}

/// A helper struct for creating the response from types which implements `fmt::Debug`.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug {
    value: Box<fmt::Debug + Send + 'static>,
    pretty: bool,
}

impl Debug {
    /// Create an instance of `Debug` from an value whose type has an implementation of
    /// `fmt::Debug`.
    pub fn new<T>(value: T) -> Debug
    where
        T: fmt::Debug + Send + 'static,
    {
        Debug {
            value: Box::new(value),
            pretty: false,
        }
    }

    /// Set whether this responder uses the pretty-printed specifier (`"{:#?}"`) or not.
    pub fn pretty(mut self, enabled: bool) -> Self {
        self.pretty = enabled;
        self
    }
}

impl Responder for Debug {
    fn respond(self, _: &Input) -> Result<Output, Error> {
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
        response
            .headers_mut()
            .insert(header::CONTENT_LENGTH, unsafe {
                HeaderValue::from_shared_unchecked(Bytes::from(body_len.to_string()))
            });
    }
}
