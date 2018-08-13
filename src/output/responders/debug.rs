use std::fmt;
use std::mem::PinMut;

use http::Response;

use super::text::Text;
use error::Never;
use input::Input;
use output::payloads::Once;
use output::Responder;

/// A helper struct for creating the response from types which implements `fmt::Debug`.
///
/// NOTE: This wrapper is only for debugging and should not use in the production code.
#[derive(Debug)]
pub struct Debug<T> {
    value: T,
    pretty: bool,
}

impl<T: fmt::Debug> Debug<T> {
    /// Create an instance of `Debug` from an value whose type has an implementation of
    /// `fmt::Debug`.
    pub fn new(value: T) -> Debug<T> {
        Debug {
            value,
            pretty: false,
        }
    }

    /// Set whether this responder uses the pretty-printed specifier (`"{:#?}"`) or not.
    pub fn pretty(self, enabled: bool) -> Self {
        Debug {
            pretty: enabled,
            ..self
        }
    }
}

impl<T: fmt::Debug> Responder for Debug<T> {
    type Body = Once<Text<String>>;
    type Error = Never;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let body = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };
        Text(body).respond(input)
    }
}
