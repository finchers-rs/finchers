//! Error types thrown from finchers

use http::StatusCode;
use std::borrow::Cow;
use std::{error, fmt};

pub trait HttpError: error::Error + Send + 'static {
    fn status_code(&self) -> StatusCode;
}

impl HttpError for ! {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct BadRequest<E> {
    err: E,
}

impl<E> BadRequest<E> {
    pub fn new(err: E) -> Self {
        BadRequest { err }
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

impl<E: error::Error + Send + 'static> HttpError for BadRequest<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct ServerError<E> {
    err: E,
}

impl<E> ServerError<E> {
    pub fn new(err: E) -> Self {
        ServerError { err }
    }
}

impl<E: fmt::Display> fmt::Display for ServerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl<E: error::Error> error::Error for ServerError<E> {
    fn description(&self) -> &str {
        self.err.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.err.cause()
    }
}

impl<E: error::Error + Send + 'static> HttpError for ServerError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Canceled => f.write_str("no route"),
            ErrorKind::Aborted(ref e) => fmt::Display::fmt(e, f),
        }
    }
}
