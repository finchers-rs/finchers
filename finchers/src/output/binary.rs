use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Request, Response};
use std::borrow::Cow;

use super::IntoResponse;
use crate::error::Never;

impl IntoResponse for &'static [u8] {
    type Body = &'static [u8];
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl IntoResponse for Vec<u8> {
    type Body = Vec<u8>;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl IntoResponse for Cow<'static, [u8]> {
    type Body = Cow<'static, [u8]>;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl IntoResponse for Bytes {
    type Body = Bytes;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

fn make_binary_response<T>(body: T) -> Response<T> {
    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response
}
