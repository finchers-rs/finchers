use std::borrow::Cow;
use std::mem::PinMut;

use bytes::Bytes;
use http::header::HeaderValue;
use http::{header, Response};

use crate::error::Never;
use crate::input::Input;
use crate::output::payloads::Once;
use crate::output::Responder;

#[derive(Debug)]
pub struct Binary<T>(pub T);

impl<T: AsRef<[u8]>> AsRef<[u8]> for Binary<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + Send + 'static> Responder for Binary<T> {
    type Body = Once<Self>;
    type Error = Never;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(Once::new(self));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        Ok(response)
    }
}

impl Responder for &'static [u8] {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(input)
    }
}

impl Responder for Vec<u8> {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(input)
    }
}

impl Responder for Cow<'static, [u8]> {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(input)
    }
}

impl Responder for Bytes {
    type Body = Once<Binary<Self>>;
    type Error = Never;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        Binary(self).respond(input)
    }
}
