use std::borrow::Cow;
use super::{header, ResponderContext, Response, ResponseBuilder, StatusCode};



pub trait Responder {
    fn respond_to(&mut self, ctx: &mut ResponderContext) -> Response;
}


pub trait IntoResponder {
    type Responder: Responder;
    fn into_responder(self) -> Self::Responder;
}

impl<R: Responder> IntoResponder for R {
    type Responder = R;
    fn into_responder(self) -> Self::Responder {
        self
    }
}


#[derive(Debug)]
pub struct UnitResponder;

impl Responder for UnitResponder {
    fn respond_to(&mut self, _: &mut ResponderContext) -> Response {
        ResponseBuilder::default()
            .status(StatusCode::NoContent)
            .header(header::ContentLength(0))
            .finish()
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
    fn respond_to(&mut self, _: &mut ResponderContext) -> Response {
        let body = self.0.take().expect("cannot respond twice");
        ResponseBuilder::default()
            .header(header::ContentType::plaintext())
            .header(header::ContentLength(body.len() as u64))
            .body(body)
            .finish()
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
