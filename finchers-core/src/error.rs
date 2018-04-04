//! Error types thrown from finchers

use std::borrow::Cow;
use std::{error, fmt};
use std::ops::Deref;
use http::StatusCode;
use response::HttpStatus;

#[allow(missing_docs)]
pub trait HttpError: error::Error + HttpStatus {}

impl<E: error::Error + HttpStatus> HttpError for E {}

macro_rules! impl_http_error {
    (@bad_request) => { StatusCode::BAD_REQUEST };
    (@server_error) => { StatusCode::INTERNAL_SERVER_ERROR };

    ($( @$i:ident $t:ty; )*) => {$(
        impl HttpStatus for $t {
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
    @server_error ::hyper::Error;
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Error {
    inner: Box<HttpError + Send + 'static>,
}

impl<E: HttpError + Send + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Error {
            inner: Box::new(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Deref for Error {
    type Target = HttpError;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, _: &Self) -> bool {
        unreachable!()
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BadRequest<E> {
    err: E,
    message: Option<Cow<'static, str>>,
}

impl<E> BadRequest<E> {
    #[allow(missing_docs)]
    pub fn new(err: E) -> Self {
        BadRequest { err, message: None }
    }

    #[allow(missing_docs)]
    pub fn with_message<S: Into<Cow<'static, str>>>(self, message: S) -> Self {
        BadRequest {
            message: Some(message.into()),
            ..self
        }
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

impl<E: error::Error + 'static> HttpStatus for BadRequest<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
    #[allow(missing_docs)]
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

impl HttpStatus for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
