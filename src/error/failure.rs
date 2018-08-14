use super::HttpError;
use failure::{Error, Fail};
use http::StatusCode;

#[derive(Debug, Fail)]
#[fail(display = "{}", cause)]
struct Failure {
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

#[allow(missing_docs)]
pub fn bad_request(err: impl Into<Error>) -> super::Error {
    Failure::new(StatusCode::BAD_REQUEST, err).into()
}

#[allow(missing_docs)]
pub fn internal_server_error(err: impl Into<Error>) -> super::Error {
    Failure::new(StatusCode::INTERNAL_SERVER_ERROR, err).into()
}
