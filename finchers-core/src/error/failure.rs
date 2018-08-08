#![allow(missing_docs)]

use super::HttpError;
use failure::{Error, Fail};
use http::StatusCode;

/// An HTTP error which represents `400 Bad Request`.
#[derive(Debug, Fail)]
#[fail(display = "{}", cause)]
pub struct Failure {
    status: StatusCode,
    cause: Error,
}

impl Failure {
    pub fn new(status: StatusCode, cause: impl Into<Error>) -> Failure {
        Failure {
            status,
            cause: cause.into(),
        }
    }
}

impl HttpError for Failure {
    fn status_code(&self) -> StatusCode {
        self.status
    }
}
