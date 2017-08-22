//! Definitions and reexports related to HTTP response

use std::fmt;
use std::error;
use hyper::StatusCode;


pub use hyper::Response;


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// The error type during `respond()`
    type Error: error::Error + Send + 'static;

    /// Convert itself to `hyper::Response`
    fn respond(self) -> Result<Response, Self::Error>;
}

impl Responder for Response {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(self)
    }
}

impl Responder for () {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_status(StatusCode::NoContent))
    }
}

impl Responder for &'static str {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_body(self))
    }
}

impl Responder for String {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_body(self))
    }
}


/// A wrapper of responders, to represents the status `201 Created`
pub struct Created<T>(pub T);

impl<T: Responder> Responder for Created<T> {
    type Error = T::Error;
    fn respond(self) -> Result<Response, Self::Error> {
        self.0
            .respond()
            .map(|res| res.with_status(StatusCode::Created))
    }
}


#[doc(hidden)]
#[derive(Debug)]
pub enum NeverReturn {}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl error::Error for NeverReturn {
    fn description(&self) -> &str {
        ""
    }
}
