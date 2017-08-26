//! Definitions and reexports related to HTTP response

use std::fmt;
use std::error;
use hyper::StatusCode;
use hyper::header;

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
        Ok(
            Response::new()
                .with_status(StatusCode::NoContent)
                .with_header(header::ContentLength(0)),
        )
    }
}

impl Responder for &'static str {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(
            Response::new()
                .with_header(header::ContentType::plaintext())
                .with_header(header::ContentLength(self.as_bytes().len() as u64))
                .with_body(self),
        )
    }
}

impl Responder for String {
    type Error = NeverReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(
            Response::new()
                .with_header(header::ContentType::plaintext())
                .with_header(header::ContentLength(self.as_bytes().len() as u64))
                .with_body(self),
        )
    }
}


/// A wrapper of responders, to represents the status `201 Created`
#[derive(Debug)]
pub struct Created<T>(pub T);

impl<T: Responder> Responder for Created<T> {
    type Error = T::Error;
    fn respond(self) -> Result<Response, Self::Error> {
        self.0
            .respond()
            .map(|res| res.with_status(StatusCode::Created))
    }
}


/// A responder represents the status `204 No Content`
#[derive(Debug)]
pub struct NoContent;

impl Responder for NoContent {
    type Error = NeverReturn;
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_status(StatusCode::NoContent))
    }
}


/// A wrapper of responders, to overwrite the value of `ContentType`.
#[derive(Debug)]
pub struct ContentType<T>(header::ContentType, T);

impl<T: Responder> ContentType<T> {
    /// Create a new instance of `ContentType`
    pub fn new(content_type: header::ContentType, responder: T) -> Self {
        ContentType(content_type, responder)
    }
}

impl<T: Responder> Responder for ContentType<T> {
    type Error = T::Error;
    fn respond(self) -> Result<Response, Self::Error> {
        let Self { 0: c, 1: res } = self;
        res.respond().map(|res| res.with_header(c))
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
