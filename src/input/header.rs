//! Components for parsing header values.

use failure::Fail;
use http::header::HeaderValue;

/// Trait representing the conversion from an entry of HTTP header.
pub trait FromHeader: Sized {
    /// The error type which will be returned from `from_header`.
    type Error: Fail;

    /// The name of header associated with this type.
    const HEADER_NAME: &'static str;

    /// Perform conversion from a byte sequence to a value of `Self`.
    fn from_header(value: &HeaderValue) -> Result<Self, Self::Error>;
}
