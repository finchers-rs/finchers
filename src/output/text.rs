use http::header::HeaderValue;
use http::{header, Response};
use std::borrow::Cow;

use super::body::{Once, Payload};
use super::{Output, OutputContext};
use error::Never;

#[doc(hidden)]
#[deprecated(since = "0.12.0-alpha.7")]
#[allow(deprecated)]
#[derive(Debug)]
pub struct Text<T>(pub T);

#[allow(deprecated)]
impl<T: AsRef<str>> AsRef<[u8]> for Text<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

#[allow(deprecated)]
impl<T> Output for Text<T>
where
    T: AsRef<str> + Send + 'static,
{
    type Body = Payload<Once<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(Payload::from(Once::new(self))))
    }
}

impl Output for &'static str {
    type Body = &'static str;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(self))
    }
}

impl Output for String {
    type Body = String;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_text_response(self))
    }
}

impl Output for Cow<'static, str> {
    type Body = Cow<'static, str>;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
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
