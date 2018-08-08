//! Error primitives.

mod failure;

pub use self::failure::Failure;

use std::borrow::Cow;
use std::fmt;

use failure::Fail;
use http::header::HeaderMap;
use http::StatusCode;

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

/// An error type indicating that a necessary elements was not given from the client.
///
/// This error value will return `400 Bad Request` as the HTTP status code.
#[derive(Debug, Fail)]
#[fail(display = "{}", message)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
    #[allow(missing_docs)]
    pub fn new<S>(message: S) -> NotPresent
    where
        S: Into<Cow<'static, str>>,
    {
        NotPresent {
            message: message.into(),
        }
    }
}

impl HttpError for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "no route")]
pub struct NoRoute;

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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl Error {
    /// Returns the reference to inner `HttpError`.
    pub fn as_http_error(&self) -> &HttpError {
        &*self.0
    }
}
