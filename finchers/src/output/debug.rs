use http::{Request, Response};
use std::fmt;

use super::IntoResponse;
use crate::error::Never;

/// An instance of `Output` representing text responses with debug output.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T>(pub T);

impl<T: fmt::Debug> IntoResponse for Debug<T> {
    type Body = String;
    type Error = Never;

    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let body = format!("{:?}", self.0);
        Ok(Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(body)
            .unwrap())
    }
}
