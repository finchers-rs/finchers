//! The definition of `ResBody` and implementors of `Payload`.

use bytes::{Buf, Bytes};
use either::Either;
use futures::{Async, Poll};
use izanami_service::http::BufStream;
use std::borrow::Cow;
use std::error;
use std::io;

use crate::error::Never;

/// The trait representing the message body in an HTTP response.
pub trait ResBody {
    /// A buffer of bytes representing a single chunk of a message body.
    type Data: Buf + Send;

    /// The error type of `Self::Payload`.
    type Error: Into<Box<dyn error::Error + Send + Sync + 'static>>;

    /// The type of `Payload` to be convert from itself.
    type Payload: BufStream<Item = Self::Data, Error = Self::Error> + Send + 'static;

    /// Converts itself into a `Payload`.
    fn into_payload(self) -> Self::Payload;
}

impl ResBody for () {
    type Data = io::Cursor<[u8; 0]>;
    type Error = Never;
    type Payload = Empty;

    fn into_payload(self) -> Self::Payload {
        Empty {
            is_end_stream: false,
        }
    }
}

impl ResBody for &'static str {
    type Data = io::Cursor<&'static str>;
    type Error = Never;
    type Payload = Once<&'static str>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl ResBody for String {
    type Data = io::Cursor<String>;
    type Error = Never;
    type Payload = Once<String>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl ResBody for Cow<'static, str> {
    type Data = io::Cursor<Cow<'static, [u8]>>;
    type Error = Never;
    type Payload = Once<Cow<'static, [u8]>>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(match self {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into()),
        })
    }
}

impl ResBody for &'static [u8] {
    type Data = io::Cursor<&'static [u8]>;
    type Error = Never;
    type Payload = Once<&'static [u8]>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl ResBody for Vec<u8> {
    type Data = io::Cursor<Vec<u8>>;
    type Error = Never;
    type Payload = Once<Vec<u8>>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl ResBody for Cow<'static, [u8]> {
    type Data = io::Cursor<Cow<'static, [u8]>>;
    type Error = Never;
    type Payload = Once<Cow<'static, [u8]>>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl ResBody for Bytes {
    type Data = io::Cursor<Bytes>;
    type Error = Never;
    type Payload = Once<Bytes>;

    fn into_payload(self) -> Self::Payload {
        Once::new2(self)
    }
}

impl<T: ResBody> ResBody for Option<T> {
    type Data = Either<T::Data, io::Cursor<[u8; 0]>>;
    type Error = T::Error;
    type Payload = Optional<T::Payload>;

    #[inline]
    fn into_payload(self) -> Self::Payload {
        optional(self.map(ResBody::into_payload))
    }
}

impl<L, R> ResBody for Either<L, R>
where
    L: ResBody,
    R: ResBody,
{
    type Data = Either<L::Data, R::Data>;
    type Error = Box<dyn error::Error + Send + Sync + 'static>;
    type Payload = EitherPayload<L::Payload, R::Payload>;

    #[inline]
    fn into_payload(self) -> Self::Payload {
        either(
            self.map_left(ResBody::into_payload)
                .map_right(ResBody::into_payload),
        )
    }
}

/// A wrapper struct for providing the implementation of `ResBody` for `T: Payload`.
#[derive(Debug)]
pub struct Payload<T>(T);

impl<T: BufStream> From<T> for Payload<T> {
    fn from(data: T) -> Self {
        Payload(data)
    }
}

impl<T> ResBody for Payload<T>
where
    T: BufStream + Send + 'static,
    T::Item: Send,
    T::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
{
    type Data = T::Item;
    type Error = T::Error;
    type Payload = T;

    fn into_payload(self) -> Self::Payload {
        self.0
    }
}

/// An instance of `Payload` representing a sized, empty message body.
#[derive(Debug)]
pub struct Empty {
    is_end_stream: bool,
}

impl BufStream for Empty {
    type Item = io::Cursor<[u8; 0]>;
    type Error = Never;

    fn poll_buf(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.is_end_stream {
            self.is_end_stream = true;
            Ok(Async::Ready(Some(io::Cursor::new([]))))
        } else {
            Ok(Async::Ready(None))
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_end_stream
    }
}

/// A `Payload` representing a sized data.
#[derive(Debug)]
pub struct Once<T>(Option<T>);

impl<T> Once<T> {
    #[inline]
    fn new2(data: T) -> Once<T> {
        Once(Some(data))
    }
}

impl<T> BufStream for Once<T>
where
    T: AsRef<[u8]> + Send + 'static,
{
    type Item = io::Cursor<T>;
    type Error = Never;

    fn poll_buf(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(self.0.take().map(io::Cursor::new)))
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_none()
    }
}

/// An instance of `Payload` which acts either a certain data or an empty bytes.
#[derive(Debug)]
pub struct Optional<T>(Either<T, bool>);

impl<T: BufStream> BufStream for Optional<T> {
    type Item = Either<T::Item, io::Cursor<[u8; 0]>>;
    type Error = T::Error;

    fn poll_buf(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.0 {
            Either::Left(ref mut payload) => payload.poll_buf().map_ok_async_some(Either::Left),
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

    fn is_end_stream(&self) -> bool {
        match self.0 {
            Either::Left(ref payload) => payload.is_end_stream(),
            Either::Right(is_end_stream) => is_end_stream,
        }
    }
}

// currently not a public API.
#[doc(hidden)]
pub fn optional<T>(bd: Option<T>) -> Optional<T> {
    match bd {
        Some(bd) => Optional(Either::Left(bd)),
        None => Optional(Either::Right(false)),
    }
}

/// An instance of `Payload` which acts either one of the two `Payload`s.
#[derive(Debug)]
pub struct EitherPayload<L, R>(Either<L, R>);

impl<L, R> BufStream for EitherPayload<L, R>
where
    L: BufStream,
    R: BufStream,
    L::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    R::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
{
    type Item = Either<L::Item, R::Item>;
    type Error = Box<dyn error::Error + Send + Sync + 'static>;

    #[inline]
    fn poll_buf(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.0 {
            Either::Left(ref mut left) => left
                .poll_buf()
                .map_ok_async_some(Either::Left)
                .map_err(Into::into),
            Either::Right(ref mut right) => right
                .poll_buf()
                .map_ok_async_some(Either::Right)
                .map_err(Into::into),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match self.0 {
            Either::Left(ref left) => left.is_end_stream(),
            Either::Right(ref right) => right.is_end_stream(),
        }
    }
}

// currently not a public API.
#[doc(hidden)]
pub fn either<L, R>(either: Either<L, R>) -> EitherPayload<L, R> {
    EitherPayload(either)
}

trait PollExt<T, E> {
    fn map_ok_async_some<U>(self, f: impl FnOnce(T) -> U) -> Poll<Option<U>, E>;
}

impl<T, E> PollExt<T, E> for Poll<Option<T>, E> {
    fn map_ok_async_some<U>(self, f: impl FnOnce(T) -> U) -> Poll<Option<U>, E> {
        self.map(|x_async| x_async.map(|x_opt| x_opt.map(f)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Buf;
    use futures::stream;
    use futures::Stream;
    use tokio::runtime::current_thread::Runtime;

    #[derive(Debug)]
    struct PayloadData {
        data: Vec<Bytes>,
    }

    fn run_body(body: impl ResBody) -> PayloadData {
        let payload = &mut body.into_payload();

        let mut rt = Runtime::new().expect("failed to initialize the test runtime");
        let data = rt
            .block_on(stream::poll_fn(|| payload.poll_buf()).collect())
            .map_err(Into::into)
            .expect("failed to collect the payload data");

        PayloadData {
            data: data.into_iter().map(|chunk| chunk.collect()).collect(),
        }
    }

    #[test]
    fn test_empty() {
        let payload = run_body(());
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
    }

    #[test]
    fn test_optional_some() {
        let payload = run_body(Some("Alice"));
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 5);
    }

    #[test]
    fn test_optional_none() {
        let payload = run_body(None as Option<&str>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
    }

    #[test]
    fn test_either() {
        let payload = run_body(Either::Left("Alice") as Either<&str, ()>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 5);

        let payload = run_body(Either::Right(()) as Either<&str, ()>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
    }
}
