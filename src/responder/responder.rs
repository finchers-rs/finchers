use std::error::Error;
use http::{Cookies, Headers, IntoBody, Response, StatusCode};
use http::header::SetCookie;
use super::ResponderContext;

pub trait Responder: Sized {
    type Body: IntoBody;

    fn status(&self) -> StatusCode {
        StatusCode::Ok
    }

    fn body(&mut self) -> Option<Self::Body> {
        None
    }

    fn headers(&self, &mut Headers) {}

    fn cookies(&self, &mut Cookies) {}
}

impl Responder for () {
    type Body = ();
    fn status(&self) -> StatusCode {
        StatusCode::NoContent
    }
}

pub trait IntoResponder {
    type Responder: Responder;
    fn into_responder(self) -> Self::Responder;
}

impl<R: Responder> IntoResponder for R {
    type Responder = Self;
    fn into_responder(self) -> Self {
        self
    }
}

#[derive(Debug)]
pub struct StringResponder(Option<::std::borrow::Cow<'static, str>>);

impl Responder for StringResponder {
    type Body = ::std::borrow::Cow<'static, str>;
    fn body(&mut self) -> Option<Self::Body> {
        self.0.take()
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

impl IntoResponder for ::std::borrow::Cow<'static, str> {
    type Responder = StringResponder;
    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self))
    }
}

pub trait ErrorResponder: Error {
    fn status(&self) -> StatusCode {
        StatusCode::InternalServerError
    }

    fn message(&self) -> Option<String> {
        Some(format!(
            "description: {}\ndetail: {}",
            Error::description(self),
            self
        ))
    }
}

mod implementors {
    use super::*;
    use std::string::{FromUtf8Error, ParseError};
    use http::HttpError;

    impl ErrorResponder for FromUtf8Error {
        fn status(&self) -> StatusCode {
            StatusCode::BadRequest
        }
    }

    impl ErrorResponder for ParseError {
        fn status(&self) -> StatusCode {
            StatusCode::BadRequest
        }
    }

    impl ErrorResponder for HttpError {}
}

impl<E: ErrorResponder> Responder for E {
    type Body = String;

    fn status(&self) -> StatusCode {
        ErrorResponder::status(self)
    }

    fn body(&mut self) -> Option<Self::Body> {
        self.message()
    }
}

pub fn respond<R: IntoResponder>(res: R, ctx: &mut ResponderContext) -> Response {
    let mut res = res.into_responder();

    let mut response = Response::new();
    response.set_status(res.status());
    if let Some(body) = res.body() {
        let body = body.into_body(response.headers_mut());
        response.set_body(body);
    }
    res.headers(response.headers_mut());

    res.cookies(&mut ctx.cookies);
    let cookies = ctx.cookies.collect_changes();
    if cookies.len() > 0 {
        response.headers_mut().set(SetCookie(cookies));
    }

    response
}
