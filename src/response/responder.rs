use std::borrow::Cow;
use hyper::{header, StatusCode};
use context::Context;
use super::Response;


/// The type to be converted to `hyper::Response`
pub trait Responder {
    /// Convert itself to `hyper::Response`
    fn respond_to(&mut self, ctx: &mut Context) -> Response;
}

/// The type to convert to a `Responder`
pub trait IntoResponder {
    type Responder: Responder;
    fn into_responder(self) -> Self::Responder;
}


#[derive(Debug)]
pub struct UnitResponder;

impl Responder for UnitResponder {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        Response::new()
            .with_status(StatusCode::NoContent)
            .with_header(header::ContentLength(0))
    }
}

impl IntoResponder for () {
    type Responder = UnitResponder;
    fn into_responder(self) -> Self::Responder {
        UnitResponder
    }
}


#[derive(Debug)]
pub struct StringResponder(Option<Cow<'static, str>>);

impl Responder for StringResponder {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        let body = self.0.take().expect("cannot respond twice");
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}

impl IntoResponder for &'static str {
    type Responder = StringResponder;
    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for String {
    type Responder = StringResponder;
    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for Cow<'static, str> {
    type Responder = StringResponder;
    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self))
    }
}
