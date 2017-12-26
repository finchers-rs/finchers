use std::fmt;
use std::error;
use std::ops::Deref;
use futures::{Poll, Stream};
use hyper;


/// The abstruction of `hyper::Chunk`, represents a piece of message body.
#[derive(Debug)]
pub struct Chunk(hyper::Chunk);

impl Deref for Chunk {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}


/// An error during receiving pieces of message body.
#[derive(Debug)]
pub struct BodyError(hyper::Error);

impl From<hyper::Error> for BodyError {
    fn from(err: hyper::Error) -> Self {
        BodyError(err)
    }
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for BodyError {
    fn description(&self) -> &str {
        self.0.description()
    }
}



/// The instance of request body.
#[derive(Default, Debug)]
pub struct Body {
    inner: hyper::Body,
}

impl Body {
    pub(crate) fn from_raw(inner: hyper::Body) -> Self {
        Body { inner }
    }
}

impl Stream for Body {
    type Item = Chunk;
    type Error = BodyError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match try_ready!(self.inner.poll()) {
            Some(item) => Ok(Some(Chunk(item)).into()),
            None => Ok(None.into()),
        }
    }
}
