//! Low level HTTP definitions from Hyper

mod from_body;
mod into_response;
mod segments;
pub(crate) mod request;

pub use hyper::{header, mime, Body, Chunk, Error, Method, Request as HyperRequest, Response, StatusCode};
pub use hyper::header::{Header, Headers};
pub use http_crate::{Extensions, Request as HttpRequest, Response as HttpResponse};

pub use self::from_body::FromBody;
pub use self::into_response::IntoResponse;
pub use self::request::Request;
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};

use std::error::Error as StdError;

#[allow(missing_docs)]
pub trait HttpError: StdError {
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
