//! Error types thrown from finchers

use std::fmt;
use std::error::Error as StdError;
use http::{Body, IntoResponse};
use http_crate::{Error, Response};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum NeverReturn {}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl StdError for NeverReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl PartialEq for NeverReturn {
    fn eq(&self, _: &Self) -> bool {
        unreachable!()
    }
}

impl IntoResponse for NeverReturn {
    fn into_response(self) -> Result<Response<Body>, Error> {
        unreachable!()
    }
}

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::HeaderError;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};
