//! Error types thrown from finchers

use std::fmt;
use std::error::Error;
use http::{FromBody, FromSegment, FromSegments, Header, StatusCode};

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

impl HttpError for NeverReturn {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

#[allow(missing_docs)]
pub trait HttpError: Error {
    fn status_code(&self) -> StatusCode;
}

macro_rules! impl_http_error_for_std {
    (@bad_request) => { StatusCode::BadRequest };
    (@server_error) => { StatusCode::InternalServerError };

    ($( @$i:ident $t:ty; )*) => {$(
        impl HttpError for $t {
            #[inline]
            fn status_code(&self) -> StatusCode {
                impl_http_error_for_std!(@$i)
            }
        }
    )*};
}

impl_http_error_for_std! {
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


// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::EmptyHeader;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};

impl<T: FromBody> HttpError for BodyError<T>
where
    T::Error: Error,
{
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}

impl<H: Header> HttpError for EmptyHeader<H> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}

impl<T: FromSegment> HttpError for ExtractPathError<T>
where
    T::Err: Error,
{
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}

impl<T: FromSegments> HttpError for ExtractPathsError<T>
where
    T::Err: Error,
{
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}
