//! Components for parsing header values.

use std::fmt;

use failure::Fail;
use http::header::{HeaderValue, ToStrError};
use hyperx::header::Header;
use mime::Mime;
use url::Url;

use crate::error::Never;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.3",
    note = "This trait is going to remove before releasing 0.12.0."
)]
pub trait FromHeader: Sized {
    type Error: Fail;
    const HEADER_NAME: &'static str;
    fn from_header(value: &HeaderValue) -> Result<Self, Self::Error>;
}

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

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.6",
    note = "use the external crate `finchers-header` instead."
)]
#[allow(deprecated)]
#[derive(Debug)]
pub struct TypedHeader<T>(pub T);

#[allow(deprecated)]
impl<T> TypedHeader<T> {
    #[allow(missing_docs)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[allow(deprecated)]
impl<T: Header> FromHeaderValue for TypedHeader<T> {
    type Error = hyperx::Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        T::parse_header(&value.as_bytes().into()).map(TypedHeader)
    }
}
