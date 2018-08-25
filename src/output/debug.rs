use http::Response;
use std::fmt;

use super::payload::Once;
use super::text::Text;
use super::{Output, OutputContext};
use crate::error::Never;

/// An instance of `Responder` representing text responses with debug output.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T>(pub T);

impl<T: fmt::Debug> Output for Debug<T> {
    type Body = Once<Text<String>>;
    type Error = Never;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let body = if cx.is_pretty() {
            format!("{:#?}", self.0)
        } else {
            format!("{:?}", self.0)
        };
        Text(body).respond(cx)
    }
}
