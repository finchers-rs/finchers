//! Error types thrown from finchers

use std::fmt;
use std::error::Error;
use hyper::header::{ContentLength, ContentType};
use http::{FromBody, FromSegment, FromSegments, IntoResponse, Response, StatusCode};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum NeverReturn {}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl Error for NeverReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl PartialEq for NeverReturn {
    fn eq(&self, _: &Self) -> bool {
        unreachable!()
    }
}

impl IntoResponse for NeverReturn {
    fn into_response(self) -> Response {
        unreachable!()
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct StdErrorResponseBuilder<'a> {
    status: StatusCode,
    error: Box<Error + 'a>,
}

#[allow(missing_docs)]
impl<'a> StdErrorResponseBuilder<'a> {
    pub fn new<E: Into<Box<Error + 'a>>>(status: StatusCode, error: E) -> Self {
        StdErrorResponseBuilder {
            status,
            error: error.into(),
        }
    }

    #[inline]
    pub fn bad_request<E: Into<Box<Error + 'a>>>(error: E) -> Self {
        Self::new(StatusCode::BadRequest, error)
    }

    #[inline]
    pub fn server_error<E: Into<Box<Error + 'a>>>(error: E) -> Self {
        Self::new(StatusCode::InternalServerError, error)
    }

    pub fn finish(self) -> Response {
        let body = format!("Error: {}", self.error.description());
        Response::new()
            .with_status(self.status)
            .with_header(ContentType::plaintext())
            .with_header(ContentLength(body.len() as u64))
            .with_body(body)
    }
}

macro_rules! impl_into_response_for_std_error {
    ($( @$i:ident $t:ty; )*) => {$(
        impl IntoResponse for $t {
            fn into_response(self) -> Response {
                StdErrorResponseBuilder::$i(self).finish()
            }
        }
    )*};
}

impl_into_response_for_std_error! {
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

#[cfg(feature = "unstable")]
impl IntoResponse for ! {
    fn into_response(self) -> Response {
        unreachable!()
    }
}

impl<T: Send> IntoResponse for ::std::sync::mpsc::SendError<T> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self).finish()
    }
}

impl<T: Send> IntoResponse for ::std::sync::mpsc::TrySendError<T> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self).finish()
    }
}

impl<T> IntoResponse for ::std::sync::PoisonError<T> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self).finish()
    }
}

impl<T> IntoResponse for ::std::sync::TryLockError<T> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self).finish()
    }
}

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::HeaderError;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};

impl<T: FromBody> IntoResponse for BodyError<T>
where
    T::Error: Error,
{
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}

impl<T: FromSegment> IntoResponse for ExtractPathError<T>
where
    T::Err: Error,
{
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}

impl<T: FromSegments> IntoResponse for ExtractPathsError<T>
where
    T::Err: Error,
{
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}
