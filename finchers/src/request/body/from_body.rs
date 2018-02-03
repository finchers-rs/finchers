use std::error::Error;
use std::string::FromUtf8Error;
use errors::NeverReturn;
use request::RequestParts;

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value returned from `from_body`.
    type Error: Error + 'static;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(req: &RequestParts) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = NeverReturn;

    fn from_body(_: &RequestParts, _: &[u8]) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Vec<u8> {
    type Error = NeverReturn;

    fn from_body(_: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(Vec::from(body))
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn from_body(_: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        String::from_utf8(body.into())
    }
}

impl<T: FromBody> FromBody for Option<T> {
    type Error = NeverReturn;

    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(T::from_body(request, body).ok())
    }
}

impl<T: FromBody> FromBody for Result<T, T::Error> {
    type Error = NeverReturn;

    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(T::from_body(request, body))
    }
}
