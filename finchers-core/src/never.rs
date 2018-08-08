use crate::error::HttpError;
use std::{error, fmt};

/// A type which has no possible values.
// FIXME: replace with futures_core::Never
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Never {}

impl Never {
    /// Consume itself and transform into an arbitrary type.
    ///
    /// NOTE: This function has never been actually called because the possible values don't exist.
    pub fn never_into<T>(self) -> T {
        match self {}
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl error::Error for Never {
    fn description(&self) -> &str {
        match *self {}
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {}
    }
}

impl HttpError for Never {}
