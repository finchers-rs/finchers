//! Implementors of `Payload`.

use futures::{self, Async};
use hyper::body::Payload;
use std::io;

pub use hyper::body::Body;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Empty;

impl Payload for Empty {
    type Data = io::Cursor<[u8; 0]>;
    type Error = io::Error;

    fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
        Ok(Async::Ready(Some(io::Cursor::new([]))))
    }

    fn content_length(&self) -> Option<u64> {
        Some(0)
    }
}

/// A `Payload` representing a sized data.
#[derive(Debug)]
pub struct Once<T>(Option<T>);

impl<T> Once<T> {
    /// Creates an `Once` from the specified data.
    pub fn new(data: T) -> Once<T> {
        Once(Some(data))
    }
}

impl<T: AsRef<[u8]> + Send + 'static> Payload for Once<T> {
    type Data = io::Cursor<T>;
    type Error = io::Error;

    fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
        Ok(Async::Ready(self.0.take().map(io::Cursor::new)))
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_none()
    }

    fn content_length(&self) -> Option<u64> {
        self.0.as_ref().map(|body| body.as_ref().len() as u64)
    }
}
