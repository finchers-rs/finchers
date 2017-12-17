use hyper::{Body, Response as RawResponse};
use super::{header, IntoResponder, Responder, ResponderContext, StatusCode};

#[derive(Debug)]
pub struct Response(RawResponse);

impl Response {
    pub(crate) fn into_raw(self) -> RawResponse {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    inner: RawResponse,
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
        Response(self.inner)
    }
}


#[derive(Debug)]
pub struct RawResponder(Option<Response>);

impl Responder for RawResponder {
    fn respond_to(&mut self, _: &mut ResponderContext) -> Response {
        self.0.take().expect("cannot respond twice")
    }
}

impl IntoResponder for Response {
    type Responder = RawResponder;
    fn into_responder(self) -> Self::Responder {
        RawResponder(Some(self))
    }
}
