//! Error types thrown from finchers

use std::fmt;
use std::error::Error;
use std::str::FromStr;
use endpoint::FromSegments;
use http::{FromBody, IntoResponse, Response, StatusCode};
use http::header::{ContentLength, ContentType};

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

#[derive(Debug)]
pub struct StdErrorResponseBuilder<E: Error> {
    status: StatusCode,
    error: E,
}

impl<E: Error> StdErrorResponseBuilder<E> {
    pub fn new(status: StatusCode, error: E) -> Self {
        StdErrorResponseBuilder { status, error }
    }

    #[inline]
    pub fn bad_request(error: E) -> Self {
        Self::new(StatusCode::BadRequest, error)
    }

    #[inline]
    pub fn server_error(error: E) -> Self {
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

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::EmptyHeader;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};

impl<T: FromBody> IntoResponse for BodyError<T>
where
    T::Error: Error,
{
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}

impl<T: FromStr> IntoResponse for ExtractPathError<T>
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
