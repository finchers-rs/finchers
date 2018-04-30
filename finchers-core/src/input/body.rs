use bytes::{BufMut, Bytes, BytesMut};
use error::HttpError;
use futures::Async::*;
use futures::{Future, Poll, Stream};
use http::StatusCode;
#[cfg(feature = "hyper")]
use hyper;
use std::ops::Deref;
use std::{fmt, mem};

/// A "future" which will be done when the all message body has been received.
#[derive(Debug)]
pub struct Data(DataState);

#[derive(Debug)]
enum DataState {
    Receiving(RequestBody, BytesMut),
    Done,
}

impl Future for Data {
    type Item = Bytes;
    type Error = BodyError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::DataState::*;
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
pub struct RequestBody {
    kind: RequestBodyKind,
}

enum RequestBodyKind {
    Empty,
    Once(Option<Bytes>),
    #[cfg(feature = "hyper")]
    Hyper(hyper::Body),
}

impl fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RequestBodyKind::*;
        match self.kind {
            Empty => f.debug_tuple("Empty").finish(),
            Once(..) => f.debug_tuple("Once").finish(),
            #[cfg(feature = "hyper")]
            Hyper(..) => f.debug_tuple("Hyper").finish(),
        }
    }
}

impl RequestBody {
    pub fn empty() -> RequestBody {
        RequestBody {
            kind: RequestBodyKind::Empty,
        }
    }

    pub fn once<T>(body: T) -> RequestBody
    where
        T: Into<Bytes>,
    {
        RequestBody {
            kind: RequestBodyKind::Once(Some(body.into())),
        }
    }

    #[cfg(feature = "hyper")]
    pub fn from_hyp(body: hyper::Body) -> RequestBody {
        RequestBody {
            kind: RequestBodyKind::Hyper(body),
        }
    }

    pub fn into_data(self) -> Data {
        // TODO: reserve the capacity of content-length
        Data(DataState::Receiving(self, BytesMut::new()))
    }
}

impl Stream for RequestBody {
    type Item = Chunk;
    type Error = BodyError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use self::RequestBodyKind::*;
        match self.kind {
            Empty => Ok(Ready(None)),
            Once(ref mut chunk) => Ok(Ready(chunk.take().map(Chunk::new))),
            #[cfg(feature = "hyper")]
            Hyper(ref mut body) => body.poll()
                .map(|async| async.map(|c| c.map(Chunk::from_hyp)))
                .map_err(|err| BodyError::Hyper(err)),
        }
    }
}

#[derive(Debug)]
pub struct Chunk(ChunkType);

#[derive(Debug)]
enum ChunkType {
    Shared(Bytes),
    #[cfg(feature = "hyper")]
    Hyper(hyper::Chunk),
}

impl Chunk {
    pub fn new<T>(chunk: T) -> Chunk
    where
        T: Into<Bytes>,
    {
        Chunk(ChunkType::Shared(chunk.into()))
    }

    #[cfg(feature = "hyper")]
    pub fn from_hyp(chunk: hyper::Chunk) -> Chunk {
        Chunk(ChunkType::Hyper(chunk))
    }
}

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        match self.0 {
            ChunkType::Shared(ref b) => b.as_ref(),
            #[cfg(feature = "hyper")]
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

#[derive(Debug, Fail)]
pub enum BodyError {
    #[cfg(feature = "hyper")]
    #[fail(display = "during receiving the chunk")]
    Hyper(hyper::Error),
}

impl HttpError for BodyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
