use std::fmt::Debug;
use serde::Serialize;
use serde_json;
use hyper::header::ContentType;

pub use hyper::Response;

pub trait Responder {
    type Error: Debug;
    fn respond(self) -> Result<Response, Self::Error>;
}

impl Responder for &'static str {
    type Error = ();
    fn respond(self) -> Result<Response, Self::Error> {
        Ok(Response::new().with_body(self))
    }
}

impl Responder for String {
    type Error = ();
    fn respond(self) -> Result<Response, ()> {
        Ok(Response::new().with_body(self))
    }
}


pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> Responder for Json<T> {
    type Error = serde_json::Error;
    fn respond(self) -> Result<Response, Self::Error> {
        let body = serde_json::to_string(&self.0)?;
        Ok(Response::new().with_header(ContentType::json()).with_body(
            body,
        ))
    }
}
