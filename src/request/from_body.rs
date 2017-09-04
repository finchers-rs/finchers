use std::string::FromUtf8Error;
use hyper::{header, mime};
use util::NoReturn;
use super::Request;

/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    #[allow(missing_docs)]
    type Error;

    #[allow(missing_docs)]
    fn check_request(req: &Request) -> bool;

    /// Convert the content of `body` to its type
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}


impl FromBody for Vec<u8> {
    type Error = NoReturn;

    fn check_request(req: &Request) -> bool {
        match req.header() {
            Some(&header::ContentType(ref mime)) if *mime == mime::APPLICATION_OCTET_STREAM => true,
            _ => false,
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn check_request(req: &Request) -> bool {
        match req.header() {
            Some(&header::ContentType(ref mime)) if *mime == mime::TEXT_PLAIN_UTF_8 => true,
            _ => false,
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
