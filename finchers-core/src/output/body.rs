use bytes::Bytes;
use crate::poll::Poll;
use futures::Stream;
use std::{fmt, io};

/// An asynchronous stream representing the body of HTTP response.
pub struct ResponseBody {
    inner: Inner,
}

enum Inner {
    Empty,
    Once(Option<Bytes>),
    Stream(Box<Stream<Item = Bytes, Error = io::Error> + Send>),
}

impl fmt::Debug for ResponseBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Inner::Empty => f.debug_tuple("Empty").finish(),
            Inner::Once(ref bytes) => f.debug_tuple("Once").field(bytes).finish(),
            Inner::Stream(..) => f.debug_tuple("Stream").finish(),
        }
    }
}

impl Default for ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody::empty()
    }
}

impl From<()> for ResponseBody {
    fn from(_: ()) -> ResponseBody {
        ResponseBody::empty()
    }
}

macro_rules! impl_from_once {
    ($($t:ty),*) => {$(
        impl From<$t> for ResponseBody {
            fn from(body: $t) -> ResponseBody {
                ResponseBody::once(body)
            }
        }
    )*};
}

impl_from_once!(&'static str, String, &'static [u8], Vec<u8>, Bytes);

impl ResponseBody {
    /// Create an instance of empty `ResponseBody`.
    pub fn empty() -> ResponseBody {
        ResponseBody {
            inner: Inner::Empty,
        }
    }

    /// Create an instance of `ResponseBody` from a chunk of bytes.
    pub fn once<T>(body: T) -> ResponseBody
    where
        T: Into<Bytes>,
    {
        ResponseBody {
            inner: Inner::Once(Some(body.into())),
        }
    }

    /// Create an instance of `ResponseBody` from a `Stream` of bytes.
    pub fn wrap_stream<T>(stream: T) -> ResponseBody
    where
        T: Stream<Item = Bytes, Error = io::Error> + Send + 'static,
    {
        ResponseBody {
            inner: Inner::Stream(Box::new(stream)),
        }
    }

    /// Return the length of bytes if available.
    pub fn len(&self) -> Option<usize> {
        match self.inner {
            Inner::Empty => Some(0),
            Inner::Once(ref chunk) => chunk.as_ref().map(|c| c.len()),
            Inner::Stream(..) => None,
        }
    }

    /// Poll an element of chunk from this stream.
    pub fn poll_data(&mut self) -> Poll<io::Result<Option<Bytes>>> {
        match self.inner {
            Inner::Empty => Poll::Ready(Ok(None)),
            Inner::Once(ref mut chunk) => Poll::Ready(Ok(chunk.take())),
            Inner::Stream(ref mut stream) => stream.poll().into(),
        }
    }
}
