//! Error types thrown from finchers

use std::fmt;
use std::error::Error;

#[allow(missing_docs)]
#[derive(Debug)]
pub enum NeverReturn {}

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

impl PartialEq for NeverReturn {
    fn eq(&self, _: &Self) -> bool {
        unreachable!()
    }
}

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::EmptyHeader;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};
