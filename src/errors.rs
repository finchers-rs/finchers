//! Error types thrown from finchers

use std::borrow::Cow;
use std::fmt;
use std::error::Error;
use http::{HttpError, StatusCode};

#[allow(missing_docs)]
#[derive(Debug, Copy, PartialEq, Eq, Hash)]
pub enum NeverReturn {}

impl Clone for NeverReturn {
    fn clone(&self) -> Self {
        unreachable!()
    }
}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl Error for NeverReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl HttpError for NeverReturn {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BadRequest<E> {
    err: E,
    message: Option<Cow<'static, str>>,
}

impl<E> BadRequest<E> {
    #[allow(missing_docs)]
    pub fn new(err: E) -> Self {
        BadRequest { err, message: None }
    }

    #[allow(missing_docs)]
    pub fn with_message<S: Into<Cow<'static, str>>>(self, message: S) -> Self {
        BadRequest {
            message: Some(message.into()),
            ..self
        }
    }
}

impl<E: fmt::Display> fmt::Display for BadRequest<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl<E: Error> Error for BadRequest<E> {
    fn description(&self) -> &str {
        self.err.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.err.cause()
    }
}

impl<E: Error> HttpError for BadRequest<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
    #[allow(missing_docs)]
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

impl Error for NotPresent {
    fn description(&self) -> &str {
        "not present"
    }
}

impl HttpError for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}
