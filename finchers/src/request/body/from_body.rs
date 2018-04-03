use std::error::Error;
use std::str::Utf8Error;
use bytes::Bytes;
use errors::NeverReturn;
use request::{BytesString, Input};

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value returned from `from_body`.
    type Error: Error + 'static;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(input: &Input) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = NeverReturn;

    fn from_body(_: Bytes, _: &Input) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Bytes {
    type Error = NeverReturn;

    fn from_body(body: Bytes, _: &Input) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for BytesString {
    type Error = Utf8Error;

    fn from_body(body: Bytes, _: &Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(body)
    }
}

impl<T: FromBody> FromBody for Option<T> {
    type Error = NeverReturn;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input).ok())
    }
}

impl<T: FromBody> FromBody for Result<T, T::Error> {
    type Error = NeverReturn;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input))
    }
}
