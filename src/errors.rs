//! Definition of error types

use std::fmt;
use std::error;
use hyper::StatusCode;

error_chain! {
    types {
        FinchersError, FinchersErrorKind, FinchersResultExt, FinchersResult;
    }

    errors {
        /// An error during routing
        Routing {
            description("routing")
            display("routing")
        }

        /// A HTTP status code
        Status(s: StatusCode) {
            description("status code")
            display("status code: {:?}", s)
        }

        /// An error represents `Internal Server Error`
        ServerError(err: Box<error::Error + Send + 'static>) {
            description("internal server error")
            display("server error: {}", err)
        }

        /// An error from `Responder::respond()`
        Responder(err: Box<error::Error + Send + 'static>) {
            description("responder")
            display("responder: {}", err)
        }
    }
}


#[allow(missing_docs)]
pub trait IntoStatus {
    fn into_status(&self) -> StatusCode; // FIXME: the name of this method should be `to_status`
}

impl IntoStatus for FinchersError {
    fn into_status(&self) -> StatusCode {
        match *self.kind() {
            FinchersErrorKind::Routing => StatusCode::NotFound,
            FinchersErrorKind::Status(s) => s,
            FinchersErrorKind::ServerError(_) | FinchersErrorKind::Responder(_) | FinchersErrorKind::Msg(_) => {
                StatusCode::InternalServerError
            }
        }
    }
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct DummyError;

impl fmt::Display for DummyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "dummy error")
    }
}

impl error::Error for DummyError {
    fn description(&self) -> &str {
        "dummy error"
    }
}
