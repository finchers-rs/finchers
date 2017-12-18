use std::fmt;
use std::error::Error;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    Skipped,
    EmptyHeader,
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
            TypeMismatch => "type mismatch",
        }
    }
}
