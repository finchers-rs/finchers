use super::{Error, ErrorKind};
use bytes::{BufMut, Bytes, BytesMut};
use futures::Async::*;
use futures::{Future, Poll, Stream};
#[cfg(feature = "from_hyper")]
use hyper;
use std::ops::Deref;
use std::{fmt, io, mem};

/// A future to receive the incoming request body
#[derive(Debug)]
pub struct Body(BodyState);

#[derive(Debug)]
enum BodyState {
    Receiving(BodyStream, BytesMut),
    Done,
}

impl Future for Body {
    type Item = Bytes;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyState::*;
        match self.0 {
            Receiving(ref mut body, ref mut buf) => while let Some(item) = try_ready!(body.poll()) {
                buf.reserve(item.len());
                unsafe {
                    buf.bytes_mut().copy_from_slice(&*item);
                    buf.advance_mut(item.len());
                }
            },
            Done => panic!("cannot resolve twice"),
        }
        match mem::replace(&mut self.0, Done) {
            Receiving(_, buf) => Ok(Ready(buf.freeze())),
            Done => panic!(),
        }
    }
}

/// A raw `Stream` to receive the incoming request body
pub struct BodyStream(BodyStreamKind);

enum BodyStreamKind {
    Empty,
    Once(Option<Bytes>),
    #[allow(dead_code)]
    Stream(Box<Stream<Item = Bytes, Error = io::Error> + Send>),
    #[cfg(feature = "from_hyper")]
    Hyper(hyper::Body),
}

impl fmt::Debug for BodyStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            BodyStreamKind::Empty => f.debug_tuple("Empty").finish(),
            BodyStreamKind::Once(..) => f.debug_tuple("Once").finish(),
            BodyStreamKind::Stream(..) => f.debug_tuple("Stream").finish(),
            #[cfg(feature = "from_hyper")]
            BodyStreamKind::Hyper(..) => f.debug_tuple("Hyper").finish(),
        }
    }
}

impl Default for BodyStream {
    fn default() -> Self {
        BodyStream(BodyStreamKind::Empty)
    }
}

impl BodyStream {
    pub fn into_data(self) -> Body {
        // TODO: reserve the capacity of content-length
        Body(BodyState::Receiving(self, BytesMut::new()))
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
                BodyStream(BodyStreamKind::Once(Some(body.into())))
            }
        }
    )*};
}

impl_from_for_stream! {
    Vec<u8>; &'static [u8]; String; &'static str; Bytes;
}

#[cfg(feature = "from_hyper")]
impl From<hyper::Body> for BodyStream {
    fn from(body: hyper::Body) -> BodyStream {
        BodyStream(BodyStreamKind::Hyper(body))
    }
}

impl Stream for BodyStream {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.0 {
            BodyStreamKind::Empty => Ok(Ready(None)),
            BodyStreamKind::Once(ref mut chunk) => Ok(Ready(chunk.take().map(Chunk::new))),
            BodyStreamKind::Stream(ref mut stream) => stream
                .poll()
                .map(|async| async.map(|c| c.map(Chunk::new)))
                .map_err(|err| ErrorKind::Io(err).into()),
            #[cfg(feature = "from_hyper")]
            BodyStreamKind::Hyper(ref mut body) => body.poll()
                .map(|async| async.map(|c| c.map(Chunk::from_hyp)))
                .map_err(|err| ErrorKind::Hyper(err).into()),
        }
    }
}

#[derive(Debug)]
pub struct Chunk(ChunkType);

#[derive(Debug)]
enum ChunkType {
    Shared(Bytes),
    #[cfg(feature = "from_hyper")]
    Hyper(::hyper::Chunk),
}

impl Chunk {
    pub fn new<T>(chunk: T) -> Chunk
    where
        T: Into<Bytes>,
    {
        Chunk(ChunkType::Shared(chunk.into()))
    }

    #[cfg(feature = "from_hyper")]
    fn from_hyp(chunk: ::hyper::Chunk) -> Chunk {
        Chunk(ChunkType::Hyper(chunk))
    }
}

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        match self.0 {
            ChunkType::Shared(ref b) => b.as_ref(),
            #[cfg(feature = "from_hyper")]
            ChunkType::Hyper(ref c) => c.as_ref(),
        }
    }
}

impl Deref for Chunk {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
