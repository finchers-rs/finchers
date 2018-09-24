//! Implementors of `Payload`.

use either::Either;
use futures::{Async, Poll};
use http::header::HeaderMap;
use std::error;
use std::io;

pub use hyper::body::{Body, Payload};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Empty;

impl Payload for Empty {
    type Data = io::Cursor<[u8; 0]>;
    type Error = io::Error;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
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

impl<T> Payload for Once<T>
where
    T: AsRef<[u8]> + Send + 'static,
{
    type Data = io::Cursor<T>;
    type Error = io::Error;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        Ok(Async::Ready(self.0.take().map(io::Cursor::new)))
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_none()
    }

    fn content_length(&self) -> Option<u64> {
        self.0.as_ref().map(|body| body.as_ref().len() as u64)
    }
}

/// An instance of `Payload` which acts either a certain data or an empty bytes.
#[derive(Debug)]
pub struct Optional<T>(Either<T, bool>);

impl<T> From<T> for Optional<T> {
    fn from(data: T) -> Optional<T> {
        Optional::new(data)
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(data: Option<T>) -> Optional<T> {
        match data {
            Some(data) => Optional::new(data),
            None => Optional::empty(),
        }
    }
}

impl<T> Optional<T> {
    /// Create an empty `Optional<T>`.
    pub fn empty() -> Optional<T> {
        Optional(Either::Right(false))
    }

    /// Create an `Optional<T>` from a value of `T`.
    pub fn new(data: T) -> Optional<T> {
        Optional(Either::Left(data))
    }
}

impl<T: Payload> Payload for Optional<T> {
    type Data = Either<T::Data, io::Cursor<[u8; 0]>>;
    type Error = T::Error;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self.0 {
            Either::Left(ref mut payload) => payload
                .poll_data()
                .map(|data_async| data_async.map(|data_opt| data_opt.map(Either::Left))),
            Either::Right(ref mut is_end_stream) => {
                if *is_end_stream {
                    Ok(None.into())
                } else {
                    *is_end_stream = true;
                    Ok(Some(Either::Right(io::Cursor::new([]))).into())
                }
            }
        }
    }

    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        match self.0 {
            Either::Left(ref mut payload) => payload.poll_trailers(),
            Either::Right(..) => Ok(None.into()),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self.0 {
            Either::Left(ref payload) => payload.is_end_stream(),
            Either::Right(is_end_stream) => is_end_stream,
        }
    }

    fn content_length(&self) -> Option<u64> {
        match self.0 {
            Either::Left(ref payload) => payload.content_length(),
            Either::Right(..) => Some(0),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct EitherPayload<L, R>(Either<L, R>);

impl<L, R> From<Either<L, R>> for EitherPayload<L, R> {
    fn from(either: Either<L, R>) -> Self {
        EitherPayload(either)
    }
}

impl<L, R> EitherPayload<L, R> {
    #[allow(missing_docs)]
    pub fn left(left: L) -> EitherPayload<L, R> {
        EitherPayload(Either::Left(left))
    }

    #[allow(missing_docs)]
    pub fn right(right: R) -> EitherPayload<L, R> {
        EitherPayload(Either::Right(right))
    }
}

impl<L, R> Payload for EitherPayload<L, R>
where
    L: Payload,
    R: Payload,
{
    type Data = Either<L::Data, R::Data>;
    type Error = Box<dyn error::Error + Send + Sync + 'static>;

    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self.0 {
            Either::Left(ref mut left) => left
                .poll_data()
                .map_ok_async_some(Either::Left)
                .map_err(Into::into),
            Either::Right(ref mut right) => right
                .poll_data()
                .map_ok_async_some(Either::Right)
                .map_err(Into::into),
        }
    }

    #[inline]
    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        match self.0 {
            Either::Left(ref mut left) => left.poll_trailers().map_err(Into::into),
            Either::Right(ref mut right) => right.poll_trailers().map_err(Into::into),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match self.0 {
            Either::Left(ref left) => left.is_end_stream(),
            Either::Right(ref right) => right.is_end_stream(),
        }
    }

    #[inline]
    fn content_length(&self) -> Option<u64> {
        match self.0 {
            Either::Left(ref left) => left.content_length(),
            Either::Right(ref right) => right.content_length(),
        }
    }
}

trait PollExt<T, E> {
    fn map_ok_async_some<U>(self, f: impl FnOnce(T) -> U) -> Poll<Option<U>, E>;
}

impl<T, E> PollExt<T, E> for Poll<Option<T>, E> {
    fn map_ok_async_some<U>(self, f: impl FnOnce(T) -> U) -> Poll<Option<U>, E> {
        self.map(|x_async| x_async.map(|x_opt| x_opt.map(f)))
    }
}
