use std::fmt;
use std::error::Error;
use std::string::FromUtf8Error;
use super::{mime, Request, StatusCode};
use responder::ErrorResponder;

/// The conversion from received request body.
pub trait FromBody: Sized {
    /// The type of error value during `validate` and `from_body`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(req: &Request) -> bool {
        true
    }

    /// Check whether the conversion is available, based on the incoming request.
    ///
    /// This method will be called after the route has been established
    /// and before reading the request body is started.
    #[allow(unused_variables)]
    fn validate(req: &Request) -> bool;

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}

impl FromBody for Vec<u8> {
    type Error = ();

    fn validate(_req: &Request) -> bool {
        true
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .and_then(|m| m.get_param("charset"))
            .map_or(true, |m| m == mime::UTF_8)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum FromBodyError<E> {
    BadRequest,
    FromBody(E),
}

impl<E: fmt::Display> fmt::Display for FromBodyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FromBodyError::BadRequest => f.write_str("bad request"),
            FromBodyError::FromBody(ref e) => e.fmt(f),
        }
    }
}

impl<E: Error> Error for FromBodyError<E> {
    fn description(&self) -> &str {
        match *self {
            FromBodyError::BadRequest => "bad request",
            FromBodyError::FromBody(ref e) => e.description(),
        }
    }
}

impl<E: Error> ErrorResponder for FromBodyError<E> {
    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}
