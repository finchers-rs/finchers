#![allow(missing_docs)]

use http::{IntoBody, Response};
use super::Responder;

pub fn respond<R: Responder + ?Sized>(res: &mut R) -> Response {
    let mut response = Response::new();
    response.set_status(res.status());
    if let Some(body) = res.body() {
        let body = body.into_body(response.headers_mut());
        response.set_body(body);
    }
    res.headers(response.headers_mut());
    response
}
