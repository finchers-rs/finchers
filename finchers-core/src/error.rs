//! Error types thrown from finchers

#![allow(missing_docs)]

use http::header::{self, HeaderValue};
use http::{Response, StatusCode};
use std::borrow::Cow;
use std::{error, fmt};

pub trait HttpError: error::Error + Send + 'static {
    fn status_code(&self) -> StatusCode;
}

macro_rules! impl_http_error {
    (@bad_request) => { StatusCode::BAD_REQUEST };
    (@server_error) => { StatusCode::INTERNAL_SERVER_ERROR };

    ($( @$i:ident $t:ty; )*) => {$(
        impl HttpError for $t {
            #[inline]
            fn status_code(&self) -> StatusCode {
                impl_http_error!(@$i)
            }
        }
    )*};
}

impl_http_error! {
    @bad_request ::std::char::DecodeUtf16Error;
    @bad_request ::std::char::ParseCharError;
    @bad_request ::std::net::AddrParseError;
    @bad_request ::std::num::ParseFloatError;
    @bad_request ::std::num::ParseIntError;
    @bad_request ::std::str::Utf8Error;
    @bad_request ::std::str::ParseBoolError;
    @bad_request ::std::string::ParseError;
    @bad_request ::std::string::FromUtf8Error;
    @bad_request ::std::string::FromUtf16Error;

    @server_error ::std::cell::BorrowError;
    @server_error ::std::cell::BorrowMutError;
    @server_error ::std::env::VarError;
    @server_error ::std::fmt::Error;
    @server_error ::std::io::Error;
    @server_error ::std::sync::mpsc::RecvError;
    @server_error ::std::sync::mpsc::TryRecvError;
    @server_error ::std::sync::mpsc::RecvTimeoutError;
}

#[derive(Debug)]
pub struct Error {
    inner: Box<HttpError + Send + 'static>,
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        self.inner.status_code()
    }

    pub fn is_noroute(&self) -> bool {
        self.status_code() == StatusCode::NOT_FOUND
    }

    pub fn to_response(&self) -> Response<String> {
        let body = self.inner.to_string();
        let body_len = body.len().to_string();

        let mut response = Response::new(body);
        *response.status_mut() = self.status_code();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(body_len.into())
        });
        response
    }
}

impl<E: HttpError + Send + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Error { inner: Box::new(err) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Debug)]
pub struct BadRequest<E> {
    err: E,
}

impl<E> BadRequest<E> {
    pub fn new(err: E) -> Self {
        BadRequest { err }
    }
}

impl<E: fmt::Display> fmt::Display for BadRequest<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl<E: error::Error> error::Error for BadRequest<E> {
    fn description(&self) -> &str {
        self.err.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.err.cause()
    }
}

impl<E: error::Error + Send + 'static> HttpError for BadRequest<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct ServerError<E> {
    err: E,
}

impl<E> ServerError<E> {
    pub fn new(err: E) -> Self {
        ServerError { err }
    }
}

impl<E: fmt::Display> fmt::Display for ServerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl<E: error::Error> error::Error for ServerError<E> {
    fn description(&self) -> &str {
        self.err.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.err.cause()
    }
}

impl<E: error::Error + Send + 'static> HttpError for ServerError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
    pub fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
        NotPresent {
            message: message.into(),
        }
    }
}

impl fmt::Display for NotPresent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl error::Error for NotPresent {
    fn description(&self) -> &str {
        "not present"
    }
}

impl HttpError for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct NoRoute {
    _priv: (),
}

impl NoRoute {
    pub fn new() -> Self {
        NoRoute { _priv: () }
    }
}

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("no route")
    }
}

impl error::Error for NoRoute {
    fn description(&self) -> &str {
        "no route"
    }
}

impl HttpError for NoRoute {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
