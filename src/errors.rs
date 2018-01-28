//! Error types thrown from finchers

use std::fmt;
use std::error::Error;
use http::{HttpError, StatusCode};

#[allow(missing_docs)]
#[derive(Debug, Copy, PartialEq, Eq, Hash)]
pub enum NeverReturn {}

impl Clone for NeverReturn {
    fn clone(&self) -> Self {
        unreachable!()
    }
}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl Error for NeverReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl HttpError for NeverReturn {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::EmptyHeader;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};
