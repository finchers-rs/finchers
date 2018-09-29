use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response};
use std::borrow::Cow;

use super::{Output, OutputContext};
use error::Never;

impl Output for &'static [u8] {
    type Body = &'static [u8];
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl Output for Vec<u8> {
    type Body = Vec<u8>;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl Output for Cow<'static, [u8]> {
    type Body = Cow<'static, [u8]>;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(self))
    }
}

impl Output for Bytes {
    type Body = Bytes;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
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
