//! Components for parsing header values.

use std::fmt;

use failure;
use http::header::{HeaderValue, ToStrError};
use mime::Mime;
use url::Url;

use crate::error::Never;

/// Trait representing the conversion from a header value.
pub trait FromHeaderValue: Sized + 'static {
    /// The error type which will be returned from `from_header_value()`.
    type Error: fmt::Debug + fmt::Display + Send + Sync + 'static;

    /// Perform conversion from a header value to `Self`.
    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error>;
}

impl FromHeaderValue for HeaderValue {
    type Error = Never;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(value.clone())
    }
}

impl FromHeaderValue for String {
    type Error = ToStrError;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        value.to_str().map(ToOwned::to_owned)
    }
}

impl FromHeaderValue for Mime {
    type Error = failure::Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(value.to_str()?.parse()?)
    }
}

impl FromHeaderValue for Url {
    type Error = failure::Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(Url::parse(value.to_str()?)?)
    }
}
