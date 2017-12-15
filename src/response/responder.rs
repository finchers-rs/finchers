use hyper::{header, StatusCode};
use context::Context;
use super::Response;


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// Convert itself to `hyper::Response`
    fn respond_to(self, ctx: &mut Context) -> Response;
}

impl Responder for Response {
    fn respond_to(self, _: &mut Context) -> Response {
        self
    }
}

impl Responder for () {
    fn respond_to(self, _: &mut Context) -> Response {
        Response::new()
            .with_status(StatusCode::NoContent)
            .with_header(header::ContentLength(0))
    }
}

impl Responder for &'static str {
    fn respond_to(self, _: &mut Context) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self)
    }
}

impl Responder for String {
    fn respond_to(self, _: &mut Context) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self)
    }
}
