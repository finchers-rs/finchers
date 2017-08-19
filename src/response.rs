use std::fmt::Debug;


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
