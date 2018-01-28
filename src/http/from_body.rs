use std::string::FromUtf8Error;
use super::{mime, Request};
use errors::NeverReturn;

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value during `validate` and `from_body`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(req: &Request) -> bool {
        true
    }

    /// Check whether the conversion is available, based on the incoming request.
    ///
    /// This method will be called after the route has been established
    /// and before reading the request body is started.
    #[allow(unused_variables)]
    fn validate(req: &Request) -> bool;

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = NeverReturn;

    fn validate(_: &Request) -> bool {
        true
    }

    fn from_body(_: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Vec<u8> {
    type Error = NeverReturn;

    fn validate(_req: &Request) -> bool {
        true
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .and_then(|m| m.get_param("charset"))
            .map_or(true, |m| m == mime::UTF_8)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
