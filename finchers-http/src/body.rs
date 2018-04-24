//! Components for parsing an HTTP request body.

use bytes::Bytes;
use futures::Future;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{fmt, mem, str};

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::error::BadRequest;
use finchers_core::input::{self, RequestBody};
use finchers_core::task::{self, PollTask, Task};
use finchers_core::{HttpError, Input, Never};

/// A reference counted UTF-8 sequence.
pub struct BytesString(Bytes);

impl fmt::Debug for BytesString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for BytesString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for BytesString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Into<Bytes> for BytesString {
    fn into(self) -> Bytes {
        self.0
    }
}

impl Into<String> for BytesString {
    fn into(self) -> String {
        self.as_str().to_owned()
    }
}

impl BytesString {
    pub fn from_static(s: &'static str) -> BytesString {
        unsafe { Self::from_shared_unchecked(Bytes::from_static(s.as_bytes())) }
    }

    pub fn from_shared(bytes: Bytes) -> Result<BytesString, str::Utf8Error> {
        let _ = str::from_utf8(&*bytes)?;
        Ok(unsafe { Self::from_shared_unchecked(bytes) })
    }

    pub unsafe fn from_shared_unchecked(bytes: Bytes) -> BytesString {
        BytesString(bytes)
    }

    pub fn as_str(&self) -> &str {
        unsafe { mem::transmute::<&[u8], _>(self.0.as_ref()) }
    }

    // TODO: add method creating substrings
}

/// Creates an endpoint for taking the instance of `BodyStream`
pub fn raw_body() -> RawBody {
    RawBody { _priv: () }
}

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct RawBody {
    _priv: (),
}

impl fmt::Debug for RawBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawBody").finish()
    }
}

impl Endpoint for RawBody {
    type Item = RequestBody;
    type Task = RawBodyTask;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(RawBodyTask { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawBodyTask {
    _priv: (),
}

impl Task for RawBodyTask {
    type Output = RequestBody;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let body = cx.body().expect("cannot take a body twice");
        Ok(body.into())
    }
}

/// Creates an endpoint for parsing the incoming request body into the value of `T`
pub fn data<T>() -> Data<T>
where
    T: FromData,
    T::Error: HttpError,
{
    Data { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct Data<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Data<T> {}

impl<T> Clone for Data<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Data<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Data").finish()
    }
}

impl<T> Endpoint for Data<T>
where
    T: FromData,
    T::Error: HttpError,
{
    type Item = T;
    type Task = DataTask<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        match T::is_match(cx.input()) {
            true => Some(DataTask::Init),
            false => None,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum DataTask<T> {
    Init,
    Recv(input::Data),
    Done(PhantomData<fn() -> T>),
}

impl<T> Task for DataTask<T>
where
    T: FromData,
    T::Error: HttpError,
{
    type Output = T;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        'poll: loop {
            let next = match *self {
                DataTask::Init => {
                    let body = cx.body().expect("The body has already taken");
                    DataTask::Recv(body.into_data())
                }
                DataTask::Recv(ref mut body) => {
                    let buf = try_ready!(body.poll());
                    let body = T::from_data(buf, cx.input())?;
                    return Ok(body.into());
                }
                _ => panic!("cannot resolve/reject twice"),
            };
            *self = next;
        }
    }
}

/// Trait representing the conversion from receiverd message body.
pub trait FromData: 'static + Sized {
    /// The error type returned from `from_data`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    #[allow(unused_variables)]
    fn is_match(input: &Input) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_data(body: Bytes, input: &Input) -> Result<Self, Self::Error>;
}

impl FromData for Bytes {
    type Error = Never;

    fn from_data(data: Bytes, _: &Input) -> Result<Self, Self::Error> {
        Ok(data)
    }
}

impl FromData for BytesString {
    type Error = BadRequest;

    fn from_data(data: Bytes, _: &Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(data).map_err(|e| BadRequest::new("failed to parse the message body").with_cause(e))
    }
}

impl FromData for String {
    type Error = BadRequest;

    fn from_data(data: Bytes, _: &Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(data)
            .map(Into::into)
            .map_err(|e| BadRequest::new("failed to parse the message body").with_cause(e))
    }
}

impl<T: FromData> FromData for Option<T> {
    type Error = Never;

    fn from_data(data: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_data(data, input).ok())
    }
}

impl<T: FromData> FromData for Result<T, T::Error> {
    type Error = Never;

    fn from_data(data: Bytes, input: &Input) -> Result<Self, Self::Error> {
        Ok(T::from_data(data, input))
    }
}
