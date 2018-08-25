use http::header::HeaderValue;
use http::{header, Response};
use std::borrow::Cow;

use super::payload::Once;
use super::{Output, OutputContext};
use crate::error::Never;

/// An instance of `Responder` representing UTF-8 text responses.
#[derive(Debug)]
pub struct Text<T>(pub T);

impl<T: AsRef<str>> AsRef<[u8]> for Text<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

impl<T: AsRef<str> + Send + 'static> Output for Text<T> {
    type Body = Once<Self>;
    type Error = Never;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(Once::new(self));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        Ok(response)
    }
}

impl Output for &'static str {
    type Body = Once<Text<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(cx)
    }
}

impl Output for String {
    type Body = Once<Text<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(cx)
    }
}

impl Output for Cow<'static, str> {
    type Body = Once<Text<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Text(self).respond(cx)
    }
}
