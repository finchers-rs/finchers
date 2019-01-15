use http::{Request, Response};
use std::fmt;

use super::IntoResponse;

/// An instance of `Output` representing text responses with debug output.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T>(pub T);

impl<T: fmt::Debug> IntoResponse for Debug<T> {
    type Body = String;

    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let body = format!("{:?}", self.0);
        Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(body)
            .unwrap()
    }
}
