use std::marker::PhantomData;
use futures::{Future, Poll, Stream};
use hyper;
use super::FromBody;


/// The type of a future returned from `Body::into_vec()`
#[derive(Debug)]
pub struct ParseBody<T> {
    body: hyper::Body,
    buf: Option<Vec<u8>>,
    _marker: PhantomData<T>,
}

impl<T: FromBody> ParseBody<T> {
    /// Construct a new `ParseBody<T>` from raw body stream
    pub fn new(body: hyper::Body) -> Self {
        ParseBody {
            body,
            buf: Some(Vec::new()),
            _marker: PhantomData,
        }
    }
}

impl<T: FromBody> Future for ParseBody<T> {
    type Item = T;
    type Error = ParseBodyError<T::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(item) = try_ready!(self.body.poll()) {
            if let Some(buf) = self.buf.as_mut() {
                buf.extend_from_slice(&item);
            }
        }

        let buf = self.buf.take().expect("The buffer has been already taken");
        T::from_body(buf)
            .map(Into::into)
            .map_err(ParseBodyError::Parse)
    }
}


/// The error type of `ParseBody<T>`
#[derive(Debug)]
pub enum ParseBodyError<E> {
    /// Failure occurs when it receives the body stream
    Hyper(hyper::Error),
    /// Failure occurs when it parses the request body into `T`
    Parse(E),
}

impl<T> From<hyper::Error> for ParseBodyError<T> {
    fn from(err: hyper::Error) -> Self {
        ParseBodyError::Hyper(err)
    }
}
