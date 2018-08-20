use failure::Fail;
use http::header::HeaderMap;
use http::{Method, StatusCode};
use std::fmt;

use self::EndpointErrorKind::*;
use crate::error::{Error, HttpError};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum EndpointErrorKind {
    NotMatched,
    MethodNotAllowed(Vec<Method>),
    Other(Error),
}

impl fmt::Display for EndpointErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotMatched => f.write_str("no route"),
            MethodNotAllowed(ref allowed_methods) => {
                if f.alternate() {
                    write!(
                        f,
                        "method not allowed (allowed methods: {:?})",
                        allowed_methods
                    )
                } else {
                    f.write_str("method not allowed")
                }
            }
            Other(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl HttpError for EndpointErrorKind {
    fn status_code(&self) -> StatusCode {
        match self {
            NotMatched => StatusCode::NOT_FOUND,
            MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED,
            Other(ref e) => e.status_code(),
        }
    }

    fn headers(&self, h: &mut HeaderMap) {
        match self {
            Other(ref e) => e.headers(h),
            _ => {}
        }
    }

    fn cause(&self) -> Option<&dyn Fail> {
        match self {
            Other(ref e) => e.cause(),
            _ => None,
        }
    }
}

#[allow(missing_docs)]
pub type EndpointResult<F> = Result<F, EndpointErrorKind>;
