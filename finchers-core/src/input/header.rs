use http::header::HeaderValue;
use std::string::FromUtf8Error;

/// Trait representing the conversion from an entry of HTTP header.
pub trait FromHeaderValue: Sized {
    /// The error type which will be returned from `from_header`.
    type Error;

    /// Perform conversion from a byte sequence to a value of `Self`.
    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error>;
}

impl FromHeaderValue for String {
    type Error = FromUtf8Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        String::from_utf8(value.as_bytes().to_vec())
    }
}
