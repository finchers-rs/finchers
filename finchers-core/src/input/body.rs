use bytes::{BufMut, Bytes, BytesMut};
use error::HttpError;
use futures::{Async, Stream};
use http::StatusCode;
#[cfg(feature = "hyper")]
use hyper;
use std::ops::Deref;
use std::{fmt, mem};
use task::PollTask;

/// An asynchronous task to receive the contents of message body.
#[derive(Debug)]
pub struct Data(DataState);

#[derive(Debug)]
enum DataState {
    Receiving(RequestBody, BytesMut),
    Done,
}

impl Data {
    /// Poll whether the all contents of the message body has been received or not.
    // FIXME: make adapt to the signature of futures2.
    // FIXME: should we replace with `core::task::Poll` ?
    pub fn poll_ready(&mut self) -> PollTask<Result<Bytes, BodyError>> {
        use self::DataState::*;
        let err = match self.0 {
            Receiving(ref mut body, ref mut buf) => 'receiving: loop {
                while let Some(item) = try_ready_task!(body.poll_data()) {
                    match item {
                        Ok(item) => {
                            buf.reserve(item.len());
                            unsafe {
                                buf.bytes_mut().copy_from_slice(&*item);
                                buf.advance_mut(item.len());
                            }
                        }
                        Err(err) => break 'receiving Some(err),
                    }
                }
                break 'receiving None;
            },
            Done => panic!("cannot resolve twice"),
        };

        if let Some(err) = err {
            self.0 = Done;
            return PollTask::Ready(Err(err));
        }

        match mem::replace(&mut self.0, Done) {
            Receiving(_, buf) => PollTask::Ready(Ok(buf.freeze())),
            Done => panic!(),
        }
    }
}

/// An asyncrhonous stream to receive the chunks of incoming request body.
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

    /// Poll an element of "Chunk".
    // FIXME: make adapt to the signature of futures2 or std's Async
    pub fn poll_data(&mut self) -> PollTask<Option<Result<Chunk, BodyError>>> {
        use self::RequestBodyKind::*;
        match self.kind {
            Empty => PollTask::Ready(None),
            Once(ref mut chunk) => PollTask::Ready(chunk.take().map(Chunk::new).map(Ok)),
            #[cfg(feature = "hyper")]
            Hyper(ref mut body) => match body.poll() {
                Ok(Async::Ready(Some(chunk))) => PollTask::Ready(Some(Ok(Chunk::from_hyp(chunk)))),
                Ok(Async::Ready(None)) => PollTask::Ready(None),
                Ok(Async::NotReady) => PollTask::Pending,
                Err(err) => PollTask::Ready(Some(Err(BodyError::Hyper(err)))),
            },
        }
    }

    /// Convert the instance of itself to a "Data".
    pub fn into_data(self) -> Data {
        // TODO: reserve the capacity of content-length
        Data(DataState::Receiving(self, BytesMut::new()))
    }
}

/// A chunk of bytes in the incoming message body.
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
