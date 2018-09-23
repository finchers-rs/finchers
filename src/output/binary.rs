use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response};
use std::borrow::Cow;

use super::payload::Once;
use super::{Output, OutputContext};
use error::Never;

/// An instance of `Responder` representing binary responses.
#[derive(Debug)]
pub struct Binary<T>(pub T);

impl<T: AsRef<[u8]>> AsRef<[u8]> for Binary<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + Send + 'static> Output for Binary<T> {
    type Body = Once<Self>;
    type Error = Never;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(Once::new(self));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        Ok(response)
    }
}

impl Output for &'static [u8] {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(cx)
    }
}

impl Output for Vec<u8> {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(cx)
    }
}

impl Output for Cow<'static, [u8]> {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(cx)
    }
}

impl Output for Bytes {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(cx)
    }
}
