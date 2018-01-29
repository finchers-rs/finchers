#![allow(missing_docs)]

use std::error::Error;
use errors::NeverReturn;

pub trait FromHeader: 'static + Sized {
    type Error: Error + 'static;

    fn header_name() -> &'static str;

    fn from_header(s: &[u8]) -> Result<Self, Self::Error>;
}

impl<H: FromHeader> FromHeader for Option<H> {
    type Error = NeverReturn;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s).ok())
    }
}

impl<H: FromHeader> FromHeader for Result<H, H::Error> {
    type Error = NeverReturn;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s))
    }
}

// TODO: add implementions for Hyper's headers
