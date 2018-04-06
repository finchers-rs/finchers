use error::HttpError;
use http;
use http::StatusCode;
#[cfg(feature = "from_hyper")]
use hyper;
use mime;
use std::{error, fmt, io};

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    DecodeHeaderToStr(http::header::ToStrError),
    ParseMediaType(mime::FromStrError),
    Io(io::Error),
    #[cfg(feature = "from_hyper")]
    Hyper(hyper::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::DecodeHeaderToStr(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::ParseMediaType(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::Io(ref e) => fmt::Display::fmt(e, f),
            #[cfg(feature = "from_hyper")]
            ErrorKind::Hyper(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::DecodeHeaderToStr(ref e) => e.description(),
            ErrorKind::ParseMediaType(ref e) => e.description(),
            ErrorKind::Io(ref e) => e.description(),
            #[cfg(feature = "from_hyper")]
            ErrorKind::Hyper(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.kind {
            ErrorKind::DecodeHeaderToStr(ref e) => e.cause(),
            ErrorKind::ParseMediaType(ref e) => e.cause(),
            ErrorKind::Io(ref e) => e.cause(),
            #[cfg(feature = "from_hyper")]
            ErrorKind::Hyper(ref e) => e.cause(),
        }
    }
}

impl HttpError for Error {
    fn status_code(&self) -> StatusCode {
        match self.kind {
            | ErrorKind::DecodeHeaderToStr(..) | ErrorKind::ParseMediaType(..) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
