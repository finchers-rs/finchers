use std::borrow::Cow;
use hyper::Response;
use super::{header, ResponseBuilder, StatusCode};


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// Convert itself to `hyper::Response`
    fn respond(self) -> Response;
}

impl Responder for () {
    fn respond(self) -> Response {
        ResponseBuilder::default()
            .status(StatusCode::NoContent)
            .header(header::ContentLength(0))
            .finish()
    }
}

impl<'a> Responder for &'a str {
    fn respond(self) -> Response {
        ResponseBuilder::default()
            .header(header::ContentType::plaintext())
            .header(header::ContentLength(self.len() as u64))
            .body(self.to_owned())
            .finish()
    }
}

impl Responder for String {
    fn respond(self) -> Response {
        ResponseBuilder::default()
            .header(header::ContentType::plaintext())
            .header(header::ContentLength(self.len() as u64))
            .body(self)
            .finish()
    }
}

impl<'a> Responder for Cow<'a, str> {
    fn respond(self) -> Response {
        ResponseBuilder::default()
            .header(header::ContentType::plaintext())
            .header(header::ContentLength(self.len() as u64))
            .body(self.into_owned())
            .finish()
    }
}
