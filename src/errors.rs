use std::fmt;
use std::error;
use context::Context;
use request::Body;
use hyper::StatusCode;

error_chain! {
    types {
        FinchersError, FinchersErrorKind, FinchersResultExt, FinchersResult;
    }

    errors {
        Routing {
            description("routing")
            display("routing")
        }

        Status(s: StatusCode) {
            description("status code")
            display("status code: {:?}", s)
        }

        ServerError(err: Box<error::Error + Send + 'static>) {
            description("internal server error")
            display("server error: {}", err)
        }

        Responder(err: Box<error::Error + Send + 'static>) {
            description("responder")
            display("responder: {}", err)
        }
    }
}

pub type EndpointResult<'r, F> = Result<(Context<'r>, Option<Body>, F), (FinchersError, Option<Body>)>;


pub trait IntoStatus {
    fn into_status(&self) -> StatusCode;
}

impl IntoStatus for FinchersError {
    fn into_status(&self) -> StatusCode {
        match *self.kind() {
            FinchersErrorKind::Routing => StatusCode::NotFound,
            FinchersErrorKind::Status(s) => s,
            FinchersErrorKind::ServerError(_) |
            FinchersErrorKind::Responder(_) |
            FinchersErrorKind::Msg(_) => StatusCode::InternalServerError,
        }
    }
}


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
