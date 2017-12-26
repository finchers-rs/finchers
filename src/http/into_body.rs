use std::borrow::Cow;
use super::{header, mime, Body, Headers};


/// Conversion into the raw HTTP response body
pub trait IntoBody: Sized {
    /// Add some response headers and then convert itself into `Body`
    fn into_body(self, h: &mut Headers) -> Body;
}

impl IntoBody for () {
    fn into_body(self, h: &mut Headers) -> Body {
        h.set(header::ContentLength(0));
        Body::default()
    }
}

impl IntoBody for Vec<u8> {
    fn into_body(self, h: &mut Headers) -> Body {
        h.set(header::ContentType(mime::APPLICATION_OCTET_STREAM));
        h.set(header::ContentLength(self.len() as u64));
        Body::from_raw(self.into())
    }
}

impl IntoBody for &'static str {
    fn into_body(self, h: &mut Headers) -> Body {
        h.set(header::ContentType(mime::TEXT_PLAIN_UTF_8));
        h.set(header::ContentLength(self.len() as u64));
        Body::from_raw(self.into())
    }
}

impl IntoBody for String {
    fn into_body(self, h: &mut Headers) -> Body {
        h.set(header::ContentType(mime::TEXT_PLAIN_UTF_8));
        h.set(header::ContentLength(self.len() as u64));
        Body::from_raw(self.into())
    }
}

impl IntoBody for Cow<'static, str> {
    fn into_body(self, h: &mut Headers) -> Body {
        h.set(header::ContentType(mime::TEXT_PLAIN_UTF_8));
        h.set(header::ContentLength(self.len() as u64));
        Body::from_raw(self.into())
    }
}
