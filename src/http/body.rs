use std::mem;
use std::string::FromUtf8Error;
use futures::{Future, Poll, Stream};
use futures::future;
use futures::Async::*;
use hyper;
use super::{Request, RequestParts};
use errors::NeverReturn;

/// A raw `Stream` to receive the incoming request body
#[derive(Debug)]
pub struct BodyStream {
    inner: hyper::Body,
}

impl From<hyper::Body> for BodyStream {
    fn from(inner: hyper::Body) -> Self {
        BodyStream { inner }
    }
}

impl Stream for BodyStream {
    type Item = hyper::Chunk;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

/// A clonable, shared future to receive the incoming request body
#[derive(Debug, Clone)]
pub struct Body {
    inner: future::Shared<BodyState>,
}

impl From<hyper::Body> for Body {
    fn from(body: hyper::Body) -> Self {
        Body {
            inner: BodyState::Receiving(body, vec![]).shared(),
        }
    }
}

impl Future for Body {
    type Item = future::SharedItem<Vec<u8>>;
    type Error = future::SharedError<hyper::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

#[derive(Debug)]
pub enum BodyState {
    Receiving(hyper::Body, Vec<u8>),
    Done,
}

impl Future for BodyState {
    type Item = Vec<u8>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyState::*;
        match mem::replace(self, Done) {
            Receiving(mut body, mut buf) => loop {
                match body.poll()? {
                    Ready(Some(item)) => {
                        buf.extend_from_slice(&*item);
                        continue;
                    }
                    Ready(None) => {
                        break Ok(Ready(buf));
                    }
                    NotReady => {
                        *self = Receiving(body, buf);
                        break Ok(NotReady);
                    }
                }
            },
            Done => panic!("cannot resolve twice"),
        }
    }
}

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value during `validate` and `from_body`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(req: &Request) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = NeverReturn;

    fn from_body(_: &RequestParts, _: &[u8]) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Vec<u8> {
    type Error = NeverReturn;

    fn from_body(_: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(Vec::from(body))
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn from_body(_: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        String::from_utf8(body.into())
    }
}

impl<T: FromBody> FromBody for Option<T> {
    type Error = NeverReturn;

    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(T::from_body(request, body).ok())
    }
}

impl<T: FromBody> FromBody for Result<T, T::Error> {
    type Error = NeverReturn;

    fn from_body(request: &RequestParts, body: &[u8]) -> Result<Self, Self::Error> {
        Ok(T::from_body(request, body))
    }
}
