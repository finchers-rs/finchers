use http::{Cookies, IntoBody, Request, Response};
use http::header::SetCookie;
use super::{IntoResponder, Responder};

#[derive(Debug)]
pub struct ResponderContext {
    pub(crate) request: Request,
    pub(crate) cookies: Cookies,
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
