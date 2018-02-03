use std::fmt;
use std::error::Error as StdError;
use futures::future;
use http::StatusCode;
use hyper;
use response::HttpStatus;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Hyper(hyper::Error),
    Shared(future::SharedError<hyper::Error>),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error {
            kind: ErrorKind::Hyper(err),
        }
    }
}

impl From<future::SharedError<hyper::Error>> for Error {
    fn from(err: future::SharedError<hyper::Error>) -> Self {
        Error {
            kind: ErrorKind::Shared(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Hyper(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::Shared(ref e) => fmt::Display::fmt(&**e, f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Hyper(ref e) => e.description(),
            ErrorKind::Shared(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self.kind {
            ErrorKind::Hyper(ref e) => e.cause(),
            ErrorKind::Shared(ref e) => e.cause(),
        }
    }
}

impl HttpStatus for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
