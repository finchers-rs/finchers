#![allow(missing_docs)]

use std::fmt;
use std::error::Error;
use std::mem;
use std::string::FromUtf8Error;
use futures::{Future, Poll, Stream};
use futures::future;
use futures::Async::*;
use http::StatusCode;
use hyper;

use errors::NeverReturn;
use request::RequestParts;
use response::HttpStatus;

/// A raw `Stream` to receive the incoming request body
#[derive(Debug, Default)]
pub struct BodyStream {
    inner: hyper::Body,
}

impl From<()> for BodyStream {
    fn from(_: ()) -> Self {
        BodyStream {
            inner: Default::default(),
        }
    }
}

macro_rules! impl_from_for_stream {
    ($($t:ty;)*) => {$(
        impl From<$t> for BodyStream {
            fn from(body: $t) -> Self {
                BodyStream {
                    inner: body.into(),
                }
            }
        }
    )*};
}

impl_from_for_stream! {
    Vec<u8>;
    &'static [u8];
    ::std::borrow::Cow<'static, [u8]>;
    String;
    &'static str;
    ::std::borrow::Cow<'static, str>;
    hyper::Chunk;
    hyper::Body;
    Option<hyper::Body>;
    ::futures::sync::mpsc::Receiver<Result<hyper::Chunk, hyper::Error>>;
}

impl Into<hyper::Body> for BodyStream {
    fn into(self) -> hyper::Body {
        self.inner
    }
}

impl Stream for BodyStream {
    type Item = BodyStreamItem;
    type Error = BodyStreamError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let inner = try_ready!(self.inner.poll());
        Ok(Ready(inner.map(|inner| BodyStreamItem { inner })))
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BodyStreamItem {
    inner: hyper::Chunk,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BodyStreamError {
    inner: hyper::Error,
}

impl From<hyper::Error> for BodyStreamError {
    fn from(inner: hyper::Error) -> Self {
        BodyStreamError { inner }
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

impl From<BodyStream> for Body {
    fn from(body: BodyStream) -> Self {
        Self::from(body.inner)
    }
}

impl Future for Body {
    type Item = BodyItem;
    type Error = BodyError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let inner = try_ready!(self.inner.poll());
        Ok(Ready(BodyItem { inner }))
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

impl ::std::ops::Deref for BodyItem {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &**self.inner
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BodyError {
    inner: future::SharedError<hyper::Error>,
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.inner).fmt(f)
    }
}

impl Error for BodyError {
    fn description(&self) -> &str {
        (*self.inner).description()
    }

    fn cause(&self) -> Option<&Error> {
        (*self.inner).cause()
    }
}

impl From<future::SharedError<hyper::Error>> for BodyError {
    fn from(inner: future::SharedError<hyper::Error>) -> Self {
        BodyError { inner }
    }
}

impl HttpStatus for BodyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value returned from `from_body`.
    type Error: Error + 'static;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(req: &RequestParts) -> bool {
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
