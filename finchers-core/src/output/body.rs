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
