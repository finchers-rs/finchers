use super::Error;
use futures::{Poll, Stream};
use hyper;
use std::borrow::Cow;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Chunk {
    inner: hyper::Chunk,
}

impl From<hyper::Chunk> for Chunk {
    fn from(chunk: hyper::Chunk) -> Self {
        Chunk { inner: chunk }
    }
}

/// A raw `Stream` to receive the incoming request body
#[derive(Debug, Default)]
pub struct BodyStream {
    inner: hyper::Body,
}

impl BodyStream {
    pub(super) fn into_inner(self) -> hyper::Body {
        self.inner
    }
}

impl From<()> for BodyStream {
    fn from(_: ()) -> Self {
        BodyStream {
            inner: Default::default(),
        }
    }
}

macro_rules! impl_from_for_stream {
    ($(
        $(#[$attr:meta])*
        $t:ty;
    )*) => {$(
        $(#[$attr])*
        impl From<$t> for BodyStream {
            fn from(body: $t) -> Self {
                BodyStream {
                    inner: body.into(),
                }
            }
        }
    )*};
}

impl_from_for_stream! {
    Vec<u8>; &'static [u8]; Cow<'static, [u8]>;
    String;  &'static str;  Cow<'static, str>;

    hyper::Body;
}

impl Stream for BodyStream {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let inner = try_ready!(self.inner.poll());
        Ok(inner.map(Into::into).into())
    }
}
