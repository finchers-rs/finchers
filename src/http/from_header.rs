#![allow(missing_docs)]

use http_crate::header::{HeaderName, HeaderValue};
use errors::NeverReturn;

pub trait FromHeader: Sized {
    type Error;

    fn header_name() -> HeaderName;

    fn from_header(src: &HeaderValue) -> Result<Self, Self::Error>;
}

impl<H: FromHeader> FromHeader for Option<H> {
    type Error = NeverReturn;

    fn header_name() -> HeaderName {
        <H as FromHeader>::header_name()
    }

    fn from_header(src: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(<H as FromHeader>::from_header(src).ok())
    }
}

impl<H: FromHeader> FromHeader for Result<H, H::Error> {
    type Error = NeverReturn;

    fn header_name() -> HeaderName {
        <H as FromHeader>::header_name()
    }

    fn from_header(src: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(<H as FromHeader>::from_header(src))
    }
}

// TODO:
// * add implemention for types in hyper::header
