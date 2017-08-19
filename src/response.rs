use std::error;
use hyper::StatusCode;
use errors::DummyError;


pub use hyper::Response;


pub trait Responder {
    type Error: error::Error + Send + 'static;
    fn respond(self) -> Result<Response, Self::Error>;
}

impl Responder for Response {
    type Error = DummyError;
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(self)
    }
}

impl Responder for () {
    type Error = DummyError;
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_status(StatusCode::NoContent))
    }
}

impl Responder for &'static str {
    type Error = DummyError;
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_body(self))
    }
}

impl Responder for String {
    type Error = DummyError;
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_body(self))
    }
}


pub struct Created<T>(pub T);

impl<T: Responder> Responder for Created<T> {
    type Error = T::Error;
    fn respond(self) -> Result<Response, Self::Error> {
        self.0.respond().map(
            |res| res.with_status(StatusCode::Created),
        )
    }
}
