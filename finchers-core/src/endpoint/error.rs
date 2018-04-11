use error::HttpError;
use http::header::{self, HeaderValue};
use http::{Response, StatusCode};
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

    pub fn to_response(&self) -> Response<String> {
        let body = self.to_string();
        let body_len = body.len().to_string();

        let mut response = Response::new(body);
        *response.status_mut() = self.status_code();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(body_len.into())
        });
        response
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
