use super::{BodyStream, Error};
use bytes::{BufMut, Bytes, BytesMut};
use futures::Async::*;
use futures::{future, Future, Poll, Stream};
use hyper;
use std::mem;

/// A clonable, shared future to receive the incoming request body
#[derive(Debug, Clone)]
pub struct Body {
    inner: future::Shared<BodyState>,
}

impl From<BodyStream> for Body {
    fn from(body: BodyStream) -> Self {
        // TODO: reserve the capacity of content-length
        Body {
            inner: BodyState::Receiving(body.into_inner(), BytesMut::new()).shared(),
        }
    }
}

impl Future for Body {
    type Item = Bytes;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let inner = try_ready!(self.inner.poll());
        Ok((*inner).clone().into())
    }
}

#[derive(Debug)]
enum BodyState {
    Receiving(hyper::Body, BytesMut),
    Done,
}

impl Future for BodyState {
    type Item = Bytes;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyState::*;
        match *self {
            Receiving(ref mut body, ref mut buf) => while let Some(item) = try_ready!(body.poll()) {
                buf.reserve(item.len());
                unsafe {
                    buf.bytes_mut().copy_from_slice(&*item);
                    buf.advance_mut(item.len());
                }
            },
            Done => panic!("cannot resolve twice"),
        }
        match mem::replace(self, Done) {
            Receiving(_, buf) => Ok(Ready(buf.freeze())),
            Done => panic!(),
        }
    }
}
