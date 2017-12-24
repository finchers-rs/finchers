use hyper::Response;
use super::{IntoBody, StatusCode};
use super::header::Headers;


pub trait Responder: Sized {
    type Body: IntoBody;

    fn status(&self) -> StatusCode {
        StatusCode::Ok
    }

    fn body(&mut self) -> Option<Self::Body> {
        None
    }

    fn headers(&self, &mut Headers) {}
}


impl Responder for () {
    type Body = ();
    fn status(&self) -> StatusCode {
        StatusCode::NoContent
    }
}

impl Responder for String {
    type Body = String;
    fn body(&mut self) -> Option<Self::Body> {
        Some(::std::mem::replace(self, String::new()))
    }
}

impl Responder for &'static str {
    type Body = &'static str;
    fn body(&mut self) -> Option<Self::Body> {
        Some(self)
    }
}

impl Responder for ::std::borrow::Cow<'static, str> {
    type Body = Self;
    fn body(&mut self) -> Option<Self> {
        Some(::std::mem::replace(self, Default::default()))
    }
}


pub fn respond<R: Responder>(mut res: R) -> Response {
    let mut response = Response::new();
    response.set_status(res.status());
    if let Some(body) = res.body() {
        body.into_body(response.headers_mut());
    }
    res.headers(response.headers_mut());
    response
}
