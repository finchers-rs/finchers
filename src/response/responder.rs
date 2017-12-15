use std::error;
use hyper::{header, StatusCode};

use util::NoReturn;
use super::Response;


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// The error type during `respond()`
    type Error: error::Error + Send + 'static;

    /// Convert itself to `hyper::Response`
    fn respond(self) -> Result<Response, Self::Error>;

    #[doc(hidden)]
    fn into_response(self) -> Response
    where
        Self: Sized,
    {
        self.respond().unwrap_or_else(|err| {
            let message = error::Error::description(&err).to_owned();
            Response::new()
                .with_status(StatusCode::InternalServerError)
                .with_header(header::ContentType::plaintext())
                .with_header(header::ContentLength(message.len() as u64))
                .with_body(message)
        })
    }
}

impl Responder for Response {
    type Error = NoReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(self)
    }
}

impl Responder for () {
    type Error = NoReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new()
            .with_status(StatusCode::NoContent)
            .with_header(header::ContentLength(0)))
    }
}

impl Responder for &'static str {
    type Error = NoReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self))
    }
}

impl Responder for String {
    type Error = NoReturn;

    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.as_bytes().len() as u64))
            .with_body(self))
    }
}
