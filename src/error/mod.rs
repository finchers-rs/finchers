//! Error primitives.

mod failure;
mod never;

pub use self::failure::{bad_request, internal_server_error};
pub use self::never::Never;

use std::error;
use std::fmt;
use std::ops::Deref;

use failure::Fail;
use http::header::{HeaderMap, HeaderValue};
use http::{header, Response, StatusCode};

use crate::generic::Either;

/// Trait representing error values from endpoints.
///
/// The types which implements this trait will be implicitly converted to an HTTP response
/// by the runtime.
pub trait HttpError: Fail {
    /// Return the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Append a set of header values to the header map.
    #[allow(unused_variables)]
    fn headers(&self, headers: &mut HeaderMap) {}
}

impl<L, R> HttpError for Either<L, R>
where
    L: HttpError + error::Error,
    R: HttpError + error::Error,
{
    fn status_code(&self) -> StatusCode {
        match self {
            Either::Left(ref t) => t.status_code(),
            Either::Right(ref t) => t.status_code(),
        }
    }

    fn headers(&self, headers: &mut HeaderMap) {
        match self {
            Either::Left(ref t) => t.headers(headers),
            Either::Right(ref t) => t.headers(headers),
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "no route")]
pub(crate) struct NoRoute;

impl HttpError for NoRoute {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

/// A type which holds a value of `HttpError` in a type-erased form.
#[derive(Debug)]
pub struct Error(Box<dyn HttpError>);

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl Deref for Error {
    type Target = dyn HttpError;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.status_code() == other.status_code()
    }
}

impl Error {
    pub(crate) fn to_response(&self) -> Response<String> {
        let mut response = Response::new(format!("{:#}", self.0));
        *response.status_mut() = self.0.status_code();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        self.0.headers(response.headers_mut());
        response
    }
}
