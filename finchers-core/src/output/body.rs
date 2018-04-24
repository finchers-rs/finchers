use bytes::Bytes;
use futures::Async::*;
use futures::{Poll, Stream};
use std::io;

pub struct Body {
    inner: Inner,
}

enum Inner {
    Empty,
    Once(Option<Bytes>),
    Stream(Box<Stream<Item = Bytes, Error = io::Error> + Send>),
}

impl Default for Body {
    fn default() -> Body {
        Body::empty()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Body {
        Body::empty()
    }
}

macro_rules! impl_from_once {
    ($($t:ty),*) => {$(
        impl From<$t> for Body {
            fn from(body: $t) -> Body {
                Body::once(body)
            }
        }
    )*};
}

impl_from_once!(&'static str, String, &'static [u8], Vec<u8>, Bytes);

impl Body {
    pub fn empty() -> Body {
        Body { inner: Inner::Empty }
    }

    pub fn once<T>(body: T) -> Body
    where
        T: Into<Bytes>,
    {
        Body {
            inner: Inner::Once(Some(body.into())),
        }
    }

    pub fn wrap_stream<T>(stream: T) -> Body
    where
        T: Stream<Item = Bytes, Error = io::Error> + Send + 'static,
    {
        Body {
            inner: Inner::Stream(Box::new(stream)),
        }
    }

    pub fn len(&self) -> Option<usize> {
        match self.inner {
            Inner::Empty => Some(0),
            Inner::Once(ref chunk) => chunk.as_ref().map(|c| c.len()),
            Inner::Stream(..) => None,
        }
    }
}

impl Stream for Body {
    type Item = Bytes;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.inner {
            Inner::Empty => Ok(Ready(None)),
            Inner::Once(ref mut chunk) => Ok(Ready(chunk.take())),
            Inner::Stream(ref mut stream) => stream.poll(),
        }
    }
}
