use http::{Response, StatusCode};
use input::Input;
use output::Body;
use std::borrow::Cow;
use std::{error, fmt};

/// Trait representing errors during handling an HTTP request.
pub trait HttpError: error::Error + Send + 'static {
    /// Returns the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode;

    /// Create an instance of "Response<Body>" from this error.
    #[allow(unused_variables)]
    fn to_response(&self, input: &Input) -> Option<Response<Body>> {
        None
    }
}

impl HttpError for ! {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct BadRequest {
    message: Cow<'static, str>,
    cause: Option<Box<error::Error + Send + 'static>>,
}

impl BadRequest {
    pub fn new<S>(message: S) -> BadRequest
    where
        S: Into<Cow<'static, str>>,
    {
        BadRequest {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause<E>(mut self, cause: E) -> BadRequest
    where
        E: error::Error + Send + 'static,
    {
        self.cause = Some(Box::new(cause));
        self
    }
}

impl fmt::Display for BadRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl error::Error for BadRequest {
    fn description(&self) -> &str {
        "bad request"
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(|e| &**e as &error::Error)
    }
}

impl HttpError for BadRequest {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct ServerError {
    message: Cow<'static, str>,
    cause: Option<Box<error::Error + Send + 'static>>,
}

impl ServerError {
    pub fn new<S>(message: S) -> ServerError
    where
        S: Into<Cow<'static, str>>,
    {
        ServerError {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause<E>(mut self, cause: E) -> ServerError
    where
        E: error::Error + Send + 'static,
    {
        self.cause = Some(Box::new(cause));
        self
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl error::Error for ServerError {
    fn description(&self) -> &str {
        "server error"
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(|e| &**e as &error::Error)
    }
}

impl HttpError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

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
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Canceled,
    Aborted(Box<HttpError>),
}

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error::aborted(err)
    }
}

impl Error {
    pub fn canceled() -> Error {
        Error {
            kind: ErrorKind::Canceled,
        }
    }

    pub fn aborted<E>(err: E) -> Error
    where
        E: HttpError,
    {
        Error {
            kind: ErrorKind::Aborted(Box::new(err)),
        }
    }

    pub fn is_canceled(&self) -> bool {
        match self.kind {
            ErrorKind::Canceled => true,
            _ => false,
        }
    }

    pub fn is_aborted(&self) -> bool {
        match self.kind {
            ErrorKind::Aborted(..) => true,
            _ => false,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self.kind {
            ErrorKind::Canceled => StatusCode::NOT_FOUND,
            ErrorKind::Aborted(ref e) => e.status_code(),
        }
    }

    pub fn to_response(&self, input: &Input) -> Option<Response<Body>> {
        match self.kind {
            ErrorKind::Canceled => None,
            ErrorKind::Aborted(ref e) => e.to_response(input),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Canceled => f.write_str("no route"),
            ErrorKind::Aborted(ref e) => fmt::Display::fmt(e, f),
        }
    }
}
