//! Definition of error types

use std::fmt;
use std::error;
use hyper::{Response, StatusCode};
use response::Responder;

error_chain! {
    types {
        FinchersError, FinchersErrorKind, FinchersResultExt, FinchersResult;
    }

    errors {
        /// 400 Bad Request
        BadRequest {
            description("bad request")
            display("bad request")
        }

        /// 404 Not Found
        NotFound {
            description("not found")
            display("not found")
        }

        /// 500 Internal Server Error
        ServerError(err: Box<error::Error + Send + 'static>) {
            description("internal server error")
            display("server error: {}", err)
        }

        /// An HTTP status code
        Status(s: StatusCode) {
            description("status code")
            display("status code: {:?}", s)
        }
    }
}


impl Responder for FinchersError {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        let status = match *self.kind() {
            FinchersErrorKind::BadRequest => StatusCode::BadRequest,
            FinchersErrorKind::NotFound => StatusCode::NotFound,
            FinchersErrorKind::ServerError(_) | FinchersErrorKind::Msg(_) => StatusCode::InternalServerError,
            FinchersErrorKind::Status(s) => s,
        };
        Ok(Response::new().with_status(status))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum NeverReturn {}

impl fmt::Display for NeverReturn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl error::Error for NeverReturn {
    fn description(&self) -> &str {
        ""
    }
}
