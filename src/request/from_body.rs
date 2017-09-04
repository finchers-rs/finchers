use std::string::FromUtf8Error;
use hyper::mime;
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
        req.media_type()
            .map_or(false, |m| *m == mime::APPLICATION_OCTET_STREAM)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(false, |m| *m == mime::TEXT_PLAIN_UTF_8)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
