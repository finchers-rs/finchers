use hyper::{Body, Response};
use super::{header, Responder, StatusCode};


#[derive(Debug, Default)]
pub struct ResponseBuilder {
    inner: Response,
}

impl ResponseBuilder {
    pub fn status(mut self, status: StatusCode) -> Self {
        self.inner.set_status(status);
        self
    }

    pub fn header<H: header::Header>(mut self, header: H) -> Self {
        self.inner.headers_mut().set(header);
        self
    }

    pub fn body<B: Into<Body>>(mut self, body: B) -> Self {
        self.inner.set_body(body);
        self
    }

    pub fn finish(self) -> Response {
        self.inner
    }
}

impl Responder for Response {
    fn respond(self) -> Response {
        self
    }
}
