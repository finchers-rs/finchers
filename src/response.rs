use hyper::Response;

pub trait Responder {
    fn respond(self) -> Response;
}

impl Responder for &'static str {
    fn respond(self) -> Response {
        Response::new().with_body(self)
    }
}

impl Responder for String {
    fn respond(self) -> Response {
        Response::new().with_body(self)
    }
}
