use bytes::Bytes;
use failure::Fail;
use futures::Async;
use http::StatusCode;
use std::fmt;
use std::mem;
use std::ops::Deref;

use crate::error::HttpError;
use crate::future::Poll;

use futures::Stream;
use hyper;

/// An asyncrhonous stream to receive the chunks of incoming request body.
pub struct RequestBody {
    kind: RequestBodyKind,
}

enum RequestBodyKind {
    Empty,
    Once(Option<Bytes>),
    Hyper(hyper::Body),
    Gone,
}

impl fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RequestBodyKind::*;
        match self.kind {
            Empty => f.debug_tuple("Empty").finish(),
            Once(..) => f.debug_tuple("Once").finish(),
            Hyper(..) => f.debug_tuple("Hyper").finish(),
            Gone => f.debug_tuple("Gone").finish(),
        }
    }
}

impl RequestBody {
    /// Create an instance of empty `RequestBody`.
    pub fn empty() -> RequestBody {
        RequestBody {
            kind: RequestBodyKind::Empty,
        }
    }

    /// Create an instance of `RequestBody` from a chunk of bytes.
    pub fn once<T>(body: T) -> RequestBody
    where
        T: Into<Bytes>,
    {
        RequestBody {
            kind: RequestBodyKind::Once(Some(body.into())),
        }
    }

    /// Create an instance of `RequestBody` from `hyper::Body`.
    pub fn from_hyp(body: hyper::Body) -> RequestBody {
        RequestBody {
            kind: RequestBodyKind::Hyper(body),
        }
    }

    /// Poll an element of `Chunk`.
    // FIXME: make adapt to the signature of futures2 or std's Async
    pub fn poll_data(&mut self) -> Poll<Result<Option<Data>, PollDataError>> {
        use self::RequestBodyKind::*;
        match self.kind {
            Empty => Poll::Ready(Ok(None)),
            Once(ref mut chunk) => Poll::Ready(Ok(chunk.take().map(Data::new))),
            Hyper(ref mut body) => match body.poll() {
                Ok(Async::Ready(chunk)) => Poll::Ready(Ok(chunk.map(Data::from_hyp))),
                Ok(Async::NotReady) => Poll::Pending,
                Err(err) => Poll::Ready(Err(PollDataError::Hyper(err))),
            },
            Gone => panic!("The request body is invalid"),
        }
    }

    #[allow(missing_docs)]
    pub fn take(&mut self) -> RequestBody {
        RequestBody {
            kind: mem::replace(&mut self.kind, RequestBodyKind::Gone),
        }
    }
}

/// A chunk of bytes in the incoming message body.
#[derive(Debug)]
pub struct Data(ChunkType);

#[derive(Debug)]
enum ChunkType {
    Shared(Bytes),
    Hyper(hyper::Chunk),
}

impl Data {
    #[allow(missing_docs)]
    pub fn new<T>(chunk: T) -> Data
    where
        T: Into<Bytes>,
    {
        Data(ChunkType::Shared(chunk.into()))
    }

    #[allow(missing_docs)]
    pub fn from_hyp(chunk: hyper::Chunk) -> Data {
        Data(ChunkType::Hyper(chunk))
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        match self.0 {
            ChunkType::Shared(ref b) => b.as_ref(),
            ChunkType::Hyper(ref c) => c.as_ref(),
        }
    }
}

impl Deref for Data {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// An error type which will returned at receiving the message body.
#[derive(Debug, Fail)]
pub enum PollDataError {
    #[allow(missing_docs)]
    #[fail(display = "during receiving the chunk")]
    Hyper(hyper::Error),

    #[doc(hidden)]
    #[fail(display = "dummy for derivation of Fail")]
    __Dummy(()),
}

impl HttpError for PollDataError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
