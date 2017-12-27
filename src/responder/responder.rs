use http::{CookieJar, Headers, IntoBody, Response, StatusCode};
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

    fn cookies(&self, &mut CookieJar) {}
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
    let cookies: Vec<_> = ctx.cookies.delta().map(|c| c.to_string()).collect();
    if cookies.len() > 0 {
        response.headers_mut().set(SetCookie(cookies));
    }

    response
}
