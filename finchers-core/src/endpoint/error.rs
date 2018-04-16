use error::HttpError;
use http::StatusCode;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Canceled,
    Aborted(Box<HttpError>),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

impl<E: HttpError + Send + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Error {
            kind: ErrorKind::Aborted(Box::new(err)),
        }
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn status_code(&self) -> StatusCode {
        match self.kind {
            ErrorKind::Canceled => StatusCode::NOT_FOUND,
            ErrorKind::Aborted(ref e) => e.status_code(),
        }
    }

    pub fn is_noroute(&self) -> bool {
        self.status_code() == StatusCode::NOT_FOUND
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Canceled => f.write_str("no route"),
            ErrorKind::Aborted(ref e) => fmt::Display::fmt(e, f),
        }
    }
}
