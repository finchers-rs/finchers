use std::string::FromUtf8Error;
use hyper::mime;
use util::NoReturn;
use super::Request;

/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    /// The error type returned from `from_body()`
    type Error;

    /// Check whether the incoming request is matched or not
    fn check_request(req: &Request) -> bool;

    /// Convert the content of `body` to its type
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}


impl FromBody for Vec<u8> {
    type Error = NoReturn;

    fn check_request(_req: &Request) -> bool {
        // req.media_type()
        //     .map_or(true, |m| *m == mime::APPLICATION_OCTET_STREAM)
        true
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn check_request(req: &Request) -> bool {
        req.media_type().map_or(true, |m| {
            m.type_() == mime::TEXT && m.subtype() == mime::PLAIN
                && m.get_param("charset").map_or(true, |m| m == mime::UTF_8)
        })
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
