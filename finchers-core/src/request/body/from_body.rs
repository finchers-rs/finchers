use bytes::Bytes;
use never::Never;
use request::{BytesString, Input};
use std::error::Error;
use std::str::Utf8Error;

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value returned from `from_body`.
    type Error: Error + Send + 'static;

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
    type Error = Never;

    fn from_body(_: Bytes, _: &Input) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Bytes {
    type Error = Never;

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

impl FromBody for String {
    type Error = Utf8Error;

    fn from_body(body: Bytes, _: &Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(body).map(Into::into)
    }
}

impl<T: FromBody> FromBody for Option<T> {
    type Error = Never;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input).ok())
    }
}

impl<T: FromBody> FromBody for Result<T, T::Error> {
    type Error = Never;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input))
    }
}
