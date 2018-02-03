use std::mem;
use std::ops::Deref;
use futures::{future, Future, Poll, Stream};
use futures::Async::*;
use hyper;
use super::{BodyStream, Error};

/// A clonable, shared future to receive the incoming request body
#[derive(Debug, Clone)]
pub struct Body {
    inner: future::Shared<BodyState>,
}

impl From<BodyStream> for Body {
    fn from(body: BodyStream) -> Self {
        Body {
            inner: BodyState::Receiving(body.into_inner(), vec![]).shared(),
        }
    }
}

impl Future for Body {
    type Item = BodyItem;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let inner = try_ready!(self.inner.poll());
        Ok(BodyItem { inner }.into())
    }
}

#[derive(Debug)]
enum BodyState {
    Receiving(hyper::Body, Vec<u8>),
    Done,
}

impl Future for BodyState {
    type Item = Vec<u8>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyState::*;
        match *self {
            Receiving(ref mut body, ref mut buf) => while let Some(item) = try_ready!(body.poll()) {
                buf.extend_from_slice(&*item);
            },
            Done => panic!("cannot resolve twice"),
        }
        match mem::replace(self, Done) {
            Receiving(_, buf) => Ok(Ready(buf)),
            Done => panic!(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BodyItem {
    inner: future::SharedItem<Vec<u8>>,
}

impl Deref for BodyItem {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<[u8]> for BodyItem {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &**self.inner
    }
}
