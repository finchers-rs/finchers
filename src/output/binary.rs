use bytes::Bytes;
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
pub struct Binary<T>(pub T);

#[allow(deprecated)]
impl<T: AsRef<[u8]>> AsRef<[u8]> for Binary<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[allow(deprecated)]
impl<T> Output for Binary<T>
where
    T: AsRef<[u8]> + Send + 'static,
{
    type Body = Payload<Once<Self>>;
    type Error = Never;

    #[inline]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(make_binary_response(Payload::from(Once::new(self))))
    }
}

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
