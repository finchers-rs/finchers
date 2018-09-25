use http::Response;
use std::fmt;

use super::{Output, OutputContext};
use error::Never;

/// An instance of `Responder` representing text responses with debug output.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T>(pub T);

impl<T: fmt::Debug> Output for Debug<T> {
    type Body = String;
    type Error = Never;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let body = if cx.is_pretty() {
            format!("{:#?}", self.0)
        } else {
            format!("{:?}", self.0)
        };
        Ok(Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(body)
            .unwrap())
    }
}
