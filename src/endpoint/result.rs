#![allow(missing_docs)]

use hyper::StatusCode;
use response::{Responder, Response};
use util::NoReturn;


/// The error type during `Endpoint::apply()`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    /// This endpoint does not matches the current request
    Skipped,
    /// The instance of requst body has already been taken
    EmptyBody,
    /// The header is not set
    EmptyHeader,
    /// The method of the current request is invalid in the endpoint
    InvalidMethod,
    /// The type of a path segment or a query parameter is not convertible to the endpoint
    TypeMismatch,
}

impl Responder for EndpointError {
    type Error = NoReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_status(StatusCode::NotFound))
    }
}


/// The return type of `Endpoint::apply()`
pub type EndpointResult<T> = Result<T, EndpointError>;
