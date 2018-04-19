use http::{Response, StatusCode};
use input::Input;
use output::Body;
use std::borrow::Cow;
use std::ops::Deref;
use std::{error, fmt};

/// Trait representing errors during handling an HTTP request.
pub trait HttpError: error::Error + Send + 'static {
    /// Returns the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode;

    /// Create an instance of "Response<Body>" from this error.
    #[allow(unused_variables)]
    fn to_response(&self, input: &Input) -> Option<Response<Body>> {
        None
    }
}

impl HttpError for ! {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct BadRequest {
    message: Cow<'static, str>,
    cause: Option<Box<error::Error + Send + 'static>>,
}

impl BadRequest {
    pub fn new<S>(message: S) -> BadRequest
    where
        S: Into<Cow<'static, str>>,
    {
        BadRequest {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause<E>(mut self, cause: E) -> BadRequest
    where
        E: error::Error + Send + 'static,
    {
        self.cause = Some(Box::new(cause));
        self
    }
}

impl fmt::Display for BadRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl error::Error for BadRequest {
    fn description(&self) -> &str {
        "bad request"
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(|e| &**e as &error::Error)
    }
}

impl HttpError for BadRequest {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct ServerError {
    message: Cow<'static, str>,
    cause: Option<Box<error::Error + Send + 'static>>,
}

impl ServerError {
    pub fn new<S>(message: S) -> ServerError
    where
        S: Into<Cow<'static, str>>,
    {
        ServerError {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause<E>(mut self, cause: E) -> ServerError
    where
        E: error::Error + Send + 'static,
    {
        self.cause = Some(Box::new(cause));
        self
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl error::Error for ServerError {
    fn description(&self) -> &str {
        "server error"
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(|e| &**e as &error::Error)
    }
}

impl HttpError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug)]
pub struct NotPresent {
    message: Cow<'static, str>,
}

impl NotPresent {
    pub fn new<S>(message: S) -> NotPresent
    where
        S: Into<Cow<'static, str>>,
    {
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

impl error::Error for NotPresent {
    fn description(&self) -> &str {
        "not present"
    }
}

impl HttpError for NotPresent {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
pub struct Error(Box<HttpError>);

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

impl Deref for Error {
    type Target = HttpError;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
