use http::header::HeaderValue;
use http::{header, Request, Response};
use std::borrow::Cow;

use super::IntoResponse;
use crate::error::Never;

impl IntoResponse for &'static str {
    type Body = &'static str;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(self))
    }
}

impl IntoResponse for String {
    type Body = String;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(self))
    }
}

impl IntoResponse for Cow<'static, str> {
    type Body = Cow<'static, str>;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(self))
    }
}

fn make_text_response<T>(body: T) -> Response<T> {
    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}
