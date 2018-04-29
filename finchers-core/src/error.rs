use either::Either;
use http::{Response, StatusCode};
use input::Input;
use output::Body;
use std::borrow::Cow;
use std::ops::Deref;
use std::{error, fmt};

/// Trait representing error values from endpoints.
pub trait HttpError: fmt::Debug + fmt::Display + Send + 'static {
    /// Return the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode;

    /// Return the "Error" representation.
    fn as_error(&self) -> Option<&error::Error> {
        None
    }

    /// Create an instance of "Response<Body>" from this error.
    #[allow(unused_variables)]
    fn to_response(&self, input: &Input) -> Option<Response<Body>> {
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

    fn as_error(&self) -> Option<&error::Error> {
        match *self {
            Either::Left(ref e) => e.as_error(),
            Either::Right(ref e) => e.as_error(),
        }
    }

    fn to_response(&self, input: &Input) -> Option<Response<Body>> {
        match *self {
            Either::Left(ref e) => e.to_response(input),
            Either::Right(ref e) => e.to_response(input),
        }
    }
}

/// An HTTP error which represents "400 Bad Request".
pub struct BadRequest {
    inner: Either<Cow<'static, str>, Box<error::Error + Send + 'static>>,
}

impl<E> From<E> for BadRequest
where
    E: error::Error + Send + 'static,
{
    fn from(err: E) -> Self {
        BadRequest::from_error(err)
    }
}

impl BadRequest {
    pub fn new<S>(message: S) -> BadRequest
    where
        S: Into<Cow<'static, str>>,
    {
        BadRequest {
            inner: Either::Left(message.into()),
        }
    }

    pub fn from_error<E>(cause: E) -> BadRequest
    where
        E: error::Error + Send + 'static,
    {
        BadRequest {
            inner: Either::Right(Box::new(cause)),
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

    fn as_error(&self) -> Option<&error::Error> {
        self.inner.as_ref().right().map(|e| &**e as &error::Error)
    }
}

/// An HTTP error which represents "500 Internal Server Error"
pub struct ServerError {
    inner: Either<Cow<'static, str>, Box<error::Error + Send + 'static>>,
}

impl<E> From<E> for ServerError
where
    E: error::Error + Send + 'static,
{
    fn from(err: E) -> Self {
        ServerError::from_error(err)
    }
}

impl ServerError {
    pub fn new<S>(message: S) -> ServerError
    where
        S: Into<Cow<'static, str>>,
    {
        ServerError {
            inner: Either::Left(message.into()),
        }
    }

    pub fn from_error<E>(cause: E) -> ServerError
    where
        E: error::Error + Send + 'static,
    {
        ServerError {
            inner: Either::Right(Box::new(cause)),
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

    fn as_error(&self) -> Option<&error::Error> {
        self.inner.as_ref().right().map(|e| &**e as &error::Error)
    }
}

/// An error type indicating that a necessary elements was not given from the client.
///
/// This error value will return "400 Bad Request" as the HTTP status code.
#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
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

/// A type which holds a value of "HttpError" in a type-erased form.
#[derive(Debug)]
pub struct Error(Box<HttpError>);

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

impl Error {
    /// Return a reference to the internal error value.
    pub fn http_error(&self) -> &HttpError {
        &*self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl Deref for Error {
    type Target = HttpError;

    fn deref(&self) -> &Self::Target {
        self.http_error()
    }
}
