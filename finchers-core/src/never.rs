#![allow(missing_docs)]

use std::{error, fmt};
use http::StatusCode;
use response::HttpStatus;

// TODO: replace with primitive never_type (!)

#[derive(Debug, Copy, PartialEq, Eq)]
pub enum Never {}

impl Clone for Never {
    fn clone(&self) -> Self {
        match *self {}
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl error::Error for Never {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl HttpStatus for Never {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}
