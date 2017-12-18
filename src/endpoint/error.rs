use std::fmt;
use std::error::Error;

/// The error type during `Endpoint::apply()`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    /// This endpoint does not matches the current request
    Skipped,
    /// The header is not set
    EmptyHeader,
    /// The method of the current request is invalid in the endpoint
    InvalidMethod,
    /// The type of a path segment or a query parameter is not convertible to the endpoint
    TypeMismatch,
}

use self::EndpointError::*;

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Error for EndpointError {
    fn description(&self) -> &str {
        match *self {
            Skipped => "skipped",
            EmptyHeader => "empty header",
            InvalidMethod => "invalid method",
            TypeMismatch => "type mismatch",
        }
    }
}
