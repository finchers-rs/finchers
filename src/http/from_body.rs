use std::string::FromUtf8Error;
use errors::NeverReturn;
use http_crate::Request;

/// The conversion from received request body.
pub trait FromBody: Sized {
    /// The type of error value during `validate` and `from_body`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match<T>(req: &Request<T>) -> bool {
        true
    }

    /// Check whether the conversion is available, based on the incoming request.
    ///
    /// This method will be called after the route has been established
    /// and before reading the request body is started.
    #[allow(unused_variables)]
    fn validate<T>(req: &Request<T>) -> bool;

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = NeverReturn;

    fn validate<T>(_: &Request<T>) -> bool {
        true
    }

    fn from_body(_: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Vec<u8> {
    type Error = NeverReturn;

    fn validate<T>(_req: &Request<T>) -> bool {
        true
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn validate<T>(_: &Request<T>) -> bool {
        true
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
