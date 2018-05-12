//! Error primitives.

use std::borrow::Cow;
use std::fmt;

use either::Either;
use failure::{self, Fail};
use http::StatusCode;
use http::header::{HeaderMap, HeaderValue};

/// Trait representing error values from endpoints.
///
/// The types which implements this trait will be implicitly converted to an HTTP response
/// by the runtime.
pub trait HttpError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    /// Return the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode;

    /// Append a set of header values to the header map.
    #[allow(unused_variables)]
    fn append_headers(&self, headers: &mut HeaderMap<HeaderValue>) {}

    /// Return the reference to a value of `Fail` if exists.
    fn as_fail(&self) -> Option<&Fail> {
        None
    }
}

impl<L, R> HttpError for Either<L, R>
where
    L: HttpError,
    R: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match *self {
            Either::Left(ref e) => e.status_code(),
            Either::Right(ref e) => e.status_code(),
        }
    }

    fn as_fail(&self) -> Option<&Fail> {
        match *self {
            Either::Left(ref e) => e.as_fail(),
            Either::Right(ref e) => e.as_fail(),
        }
    }
}

/// An HTTP error which represents `400 Bad Request`.
pub struct BadRequest {
    inner: Either<Cow<'static, str>, failure::Error>,
}

impl<E: Into<failure::Error>> From<E> for BadRequest {
    fn from(fail: E) -> Self {
        BadRequest::from_fail(fail)
    }
}

impl BadRequest {
    #[allow(missing_docs)]
    pub fn new<S>(message: S) -> BadRequest
    where
        S: Into<Cow<'static, str>>,
    {
        BadRequest {
            inner: Either::Left(message.into()),
        }
    }

    #[allow(missing_docs)]
    pub fn from_fail<E>(fail: E) -> BadRequest
    where
        E: Into<failure::Error>,
    {
        BadRequest {
            inner: Either::Right(Into::into(fail)),
        }
    }
}

impl fmt::Debug for BadRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Either::Left(ref message) => f.debug_tuple("BadRequest").field(message).finish(),
            Either::Right(ref err) => f.debug_tuple("BadRequest").field(err).finish(),
        }
    }
}

impl fmt::Display for BadRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Either::Left(ref message) => f.write_str(message),
            Either::Right(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl HttpError for BadRequest {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn as_fail(&self) -> Option<&Fail> {
        self.inner.as_ref().right().map(failure::Error::cause)
    }
}

/// An HTTP error which represents `500 Internal Server Error`
pub struct ServerError {
    inner: Either<Cow<'static, str>, failure::Error>,
}

impl<E: Into<failure::Error>> From<E> for ServerError {
    fn from(fail: E) -> Self {
        ServerError::from_fail(fail)
    }
}

impl ServerError {
    #[allow(missing_docs)]
    pub fn new<S>(message: S) -> ServerError
    where
        S: Into<Cow<'static, str>>,
    {
        ServerError {
            inner: Either::Left(message.into()),
        }
    }

    #[allow(missing_docs)]
    pub fn from_fail<E>(fail: E) -> ServerError
    where
        E: Into<failure::Error>,
    {
        ServerError {
            inner: Either::Right(Into::into(fail)),
        }
    }
}

impl fmt::Debug for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Either::Left(ref message) => f.debug_tuple("ServerError").field(message).finish(),
            Either::Right(ref err) => f.debug_tuple("ServerError").field(err).finish(),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Either::Left(ref message) => f.write_str(message),
            Either::Right(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl HttpError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn as_fail(&self) -> Option<&Fail> {
        self.inner.as_ref().right().map(failure::Error::cause)
    }
}

/// An error type indicating that a necessary elements was not given from the client.
///
/// This error value will return `400 Bad Request` as the HTTP status code.
#[derive(Debug)]
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

impl fmt::Display for NotPresent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl HttpError for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Fail)]
#[fail(display = "no route")]
struct NoRoute;

impl HttpError for NoRoute {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

/// A type which holds a value of `HttpError` in a type-erased form.
#[derive(Debug)]
pub struct Error {
    inner: Either<Box<HttpError>, NoRoute>,
}

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error {
            inner: Either::Left(Box::new(err)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Either::Left(ref e) => fmt::Display::fmt(&*e, f),
            Either::Right(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl Error {
    pub(crate) fn skipped() -> Error {
        Error {
            inner: Either::Right(NoRoute),
        }
    }

    #[allow(missing_docs)]
    pub fn is_skipped(&self) -> bool {
        self.inner.is_right()
    }

    /// Returns the reference to inner `HttpError`.
    pub fn as_http_error(&self) -> &HttpError {
        match self.inner {
            Either::Left(ref e) => &**e,
            Either::Right(ref e) => &*e as &HttpError,
        }
    }
}
