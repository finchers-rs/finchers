use std::error::Error as StdError;
use std::fmt;
use std::string::FromUtf8Error;
use hyper::mime;
use super::{Request, StatusCode};
use responder::Responder;

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
    fn validate(req: &Request) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}

impl FromBody for Vec<u8> {
    type Error = ();

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = StringBodyError;

    fn validate(req: &Request) -> Result<(), Self::Error> {
        if req.media_type()
            .and_then(|m| m.get_param("charset"))
            .map_or(true, |m| m == mime::UTF_8)
        {
            Ok(())
        } else {
            Err(StringBodyError::BadRequest)
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body).map_err(StringBodyError::FromUtf8)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum StringBodyError {
    BadRequest,
    FromUtf8(FromUtf8Error),
}

impl fmt::Display for StringBodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StringBodyError::BadRequest => f.write_str("bad request"),
            StringBodyError::FromUtf8(ref e) => e.fmt(f),
        }
    }
}

impl StdError for StringBodyError {
    fn description(&self) -> &str {
        match *self {
            StringBodyError::BadRequest => "",
            StringBodyError::FromUtf8(ref e) => e.description(),
        }
    }
}

impl Responder for StringBodyError {
    type Body = String;

    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
    }

    fn body(&mut self) -> Option<Self::Body> {
        Some(format!("{}: {}", self.description(), self))
    }
}
