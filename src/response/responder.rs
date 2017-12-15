use hyper::{header, StatusCode};
use super::Response;


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// Convert itself to `hyper::Response`
    fn respond(self) -> Response;
}

impl Responder for Response {
    fn respond(self) -> Response {
        self
    }
}

impl Responder for () {
    fn respond(self) -> Response {
        Response::new()
            .with_status(StatusCode::NoContent)
            .with_header(header::ContentLength(0))
    }
}

impl Responder for &'static str {
    fn respond(self) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self)
    }
}

impl Responder for String {
    fn respond(self) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self)
    }
}
