//! The definition of `ResBody` and implementors of `Payload`.

use bytes::{Buf, Bytes};
use either::Either;
use futures::{Async, Poll};
use http::header::HeaderMap;
use hyper;
use hyper::body::{Body, Chunk};
use std::borrow::Cow;
use std::error;
use std::io;

use error::Never;

/// The trait representing the message body in an HTTP response.
pub trait ResBody {
    /// A buffer of bytes representing a single chunk of a message body.
    type Data: Buf + Send;

    /// The error type of `Self::Payload`.
    type Error: Into<Box<dyn error::Error + Send + Sync + 'static>>;

    /// The type of `Payload` to be convert from itself.
    type Payload: hyper::body::Payload<Data = Self::Data, Error = Self::Error>;

    /// Converts itself into a `Payload`.
    fn into_payload(self) -> Self::Payload;
}

impl ResBody for Body {
    type Data = Chunk;
    type Error = hyper::Error;
    type Payload = Body;

    #[inline]
    fn into_payload(self) -> Self::Payload {
        self
    }
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

    fn into_payload(self) -> Self::Payload {
        Optional::from(self.map(ResBody::into_payload))
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
        EitherPayload::from(
            self.map_left(ResBody::into_payload)
                .map_right(ResBody::into_payload),
        )
    }
}

/// A wrapper struct for providing the implementation of `ResBody` for `T: Payload`.
#[derive(Debug)]
pub struct Payload<T: hyper::body::Payload>(T);

impl<T: hyper::body::Payload> From<T> for Payload<T> {
    fn from(data: T) -> Self {
        Payload(data)
    }
}

impl<T: hyper::body::Payload> ResBody for Payload<T> {
    type Data = T::Data;
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

impl hyper::body::Payload for Empty {
    type Data = io::Cursor<[u8; 0]>;
    type Error = Never;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
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

    fn content_length(&self) -> Option<u64> {
        Some(0)
    }
}

/// A `Payload` representing a sized data.
#[derive(Debug)]
pub struct Once<T>(Option<T>);

impl<T> Once<T> {
    /// Creates an `Once` from the specified data.
    #[doc(hidden)]
    #[deprecated(
        since = "0.12.1",
        note = "This method will be removed in the future version."
    )]
    pub fn new(data: T) -> Once<T> {
        Once(Some(data))
    }

    #[inline]
    fn new2(data: T) -> Once<T> {
        Once(Some(data))
    }
}

impl<T> hyper::body::Payload for Once<T>
where
    T: AsRef<[u8]> + Send + 'static,
{
    type Data = io::Cursor<T>;
    type Error = Never;

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

#[doc(hidden)]
#[deprecated(
    since = "0.12.1",
    note = "This impl will be removed in the future version."
)]
impl<T> From<T> for Optional<T> {
    fn from(data: T) -> Optional<T> {
        Optional(Either::Left(data))
    }
}

#[doc(hidden)]
#[deprecated(
    since = "0.12.1",
    note = "This impl will be removed in the future version."
)]
impl<T> From<Option<T>> for Optional<T> {
    fn from(data: Option<T>) -> Optional<T> {
        match data {
            Some(data) => Optional(Either::Left(data)),
            None => Optional(Either::Right(false)),
        }
    }
}

impl<T> Optional<T> {
    #[doc(hidden)]
    #[deprecated(
        since = "0.12.1",
        note = "This method will be removed in the future version."
    )]
    pub fn empty() -> Optional<T> {
        Optional(Either::Right(false))
    }

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.1",
        note = "This method will be removed in the future version."
    )]
    pub fn new(data: T) -> Optional<T> {
        Optional(Either::Left(data))
    }
}

impl<T: hyper::body::Payload> hyper::body::Payload for Optional<T> {
    type Data = Either<T::Data, io::Cursor<[u8; 0]>>;
    type Error = T::Error;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self.0 {
            Either::Left(ref mut payload) => payload.poll_data().map_ok_async_some(Either::Left),
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

/// An instance of `Payload` which acts either one of the two `Payload`s.
#[derive(Debug)]
pub struct EitherPayload<L, R>(Either<L, R>);

#[doc(hidden)]
#[deprecated(
    since = "0.12.1",
    note = "This impl will be removed in the future version."
)]
impl<L, R> From<Either<L, R>> for EitherPayload<L, R> {
    fn from(either: Either<L, R>) -> Self {
        EitherPayload(either)
    }
}

impl<L, R> EitherPayload<L, R> {
    #[doc(hidden)]
    #[deprecated(
        since = "0.12.1",
        note = "This method will be removed in the future version."
    )]
    pub fn left(left: L) -> EitherPayload<L, R> {
        EitherPayload(Either::Left(left))
    }

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.1",
        note = "This method will be removed in the future version."
    )]
    pub fn right(right: R) -> EitherPayload<L, R> {
        EitherPayload(Either::Right(right))
    }
}

impl<L, R> hyper::body::Payload for EitherPayload<L, R>
where
    L: hyper::body::Payload,
    R: hyper::body::Payload,
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

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Buf;
    use futures::future;
    use futures::stream;
    use futures::Stream;
    use hyper::body::Payload as _Payload;
    use tokio::runtime::current_thread::Runtime;

    #[derive(Debug)]
    struct PayloadData {
        data: Vec<Bytes>,
        trailers: Option<HeaderMap>,
        content_length: Option<u64>,
    }

    fn run_body(body: impl ResBody) -> PayloadData {
        let payload = &mut body.into_payload();

        let content_length = payload.content_length();
        let mut rt = Runtime::new().expect("failed to initialize the test runtime");
        let data = rt
            .block_on(stream::poll_fn(|| payload.poll_data()).collect())
            .map_err(Into::into)
            .expect("failed to collect the payload data");
        let trailers = rt
            .block_on(future::poll_fn(|| payload.poll_trailers()))
            .map_err(Into::into)
            .expect("failed to poll the trailers");

        PayloadData {
            data: data.into_iter().map(|chunk| chunk.collect()).collect(),
            trailers,
            content_length,
        }
    }

    #[test]
    fn test_empty() {
        let payload = run_body(());
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
        assert!(payload.trailers.is_none());
        assert_eq!(payload.content_length, Some(0));
    }

    #[test]
    fn test_optional_some() {
        let payload = run_body(Some("Alice"));
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 5);
        assert!(payload.trailers.is_none());
        assert_eq!(payload.content_length, Some(5));
    }

    #[test]
    fn test_optional_none() {
        let payload = run_body(None as Option<&str>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
        assert!(payload.trailers.is_none());
        assert_eq!(payload.content_length, Some(0));
    }

    #[test]
    fn test_either() {
        let payload = run_body(Either::Left("Alice") as Either<&str, ()>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 5);
        assert!(payload.trailers.is_none());
        assert_eq!(payload.content_length, Some(5));

        let payload = run_body(Either::Right(()) as Either<&str, ()>);
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].len(), 0);
        assert!(payload.trailers.is_none());
        assert_eq!(payload.content_length, Some(0));
    }
}
