//! Definition of error types

use std::error;
use hyper::{Response, StatusCode};

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


#[doc(hidden)]
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for FinchersError {
    fn into_response(self) -> Response {
        let status = match *self.kind() {
            FinchersErrorKind::BadRequest => StatusCode::BadRequest,
            FinchersErrorKind::NotFound => StatusCode::NotFound,
            FinchersErrorKind::ServerError(_) | FinchersErrorKind::Msg(_) => StatusCode::InternalServerError,
            FinchersErrorKind::Status(s) => s,
        };
        Response::new().with_status(status)
    }
}
