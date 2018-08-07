//! Components for parsing the HTTP request body.

use bytes::{Bytes, BytesMut};
use failure::Fail;
use http::StatusCode;
use std::marker::PhantomData;
use std::{fmt, mem};

use crate::endpoint::{assert_output, Context, EndpointBase};
use crate::error::BadRequest;
use crate::error::HttpError;
use crate::input::{with_get_cx, PollDataError, RequestBody};
use crate::task::Task;
use crate::{Input, Never, Poll};

/// Creates an endpoint which will take the instance of `RequestBody` from the context.
///
/// If the instance has already been stolen by another task, this endpoint will return
/// a `None`.
pub fn raw_body() -> RawBody {
    assert_output::<_, RequestBody>(RawBody { _priv: () })
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

impl EndpointBase for RawBody {
    type Output = RequestBody;
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

    fn poll_task(&mut self) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| input.body_mut().take()))
    }
}

/// Creates an endpoint which will poll the all contents of the message body
/// from the client and transform the received bytes into a value of `T`.
pub fn body<T>() -> Body<T>
where
    T: FromBody,
{
    assert_output::<_, Result<T, BodyError<T::Error>>>(Body {
        _marker: PhantomData,
    })
}

#[allow(missing_docs)]
pub struct Body<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Body<T> {}

impl<T> Clone for Body<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Body<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T> EndpointBase for Body<T>
where
    T: FromBody,
{
    type Output = Result<T, BodyError<T::Error>>;
    type Task = BodyTask<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        match T::is_match(cx.input()) {
            true => Some(BodyTask::Init(PhantomData)),
            false => None,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum BodyTask<T> {
    Init(PhantomData<fn() -> T>),
    Receiving(RequestBody, BytesMut),
    Done,
}

impl<T> Task for BodyTask<T>
where
    T: FromBody,
{
    type Output = Result<T, BodyError<T::Error>>;

    fn poll_task(&mut self) -> Poll<Self::Output> {
        use self::BodyTask::*;
        'poll: loop {
            let err = match *self {
                Init(..) => None,
                Receiving(ref mut body, ref mut buf) => 'receiving: loop {
                    let item = match poll!(body.poll_data()) {
                        Ok(Some(data)) => data,
                        Ok(None) => break 'receiving None,
                        Err(err) => break 'receiving Some(err),
                    };
                    buf.extend_from_slice(&*item);
                },
                Done => panic!("cannot resolve/reject twice"),
            };

            let ready = match (mem::replace(self, Done), err) {
                (_, Some(cause)) => Err(BodyError::Receiving(cause)),
                (Init(..), _) => {
                    let body = with_get_cx(|input| input.body_mut().take());
                    *self = Receiving(body, BytesMut::new());
                    continue 'poll;
                }
                (Receiving(_, buf), _) => {
                    with_get_cx(|input| T::from_body(buf.freeze(), input).map_err(BodyError::Parse))
                }
                _ => panic!(),
            };

            break 'poll Poll::Ready(ready);
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum BodyError<E> {
    Receiving(PollDataError),
    Parse(E),
}

impl<E: fmt::Display> fmt::Display for BodyError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BodyError::Receiving(ref e) => write!(formatter, "{}", e),
            BodyError::Parse(ref e) => write!(formatter, "{}", e),
        }
    }
}

impl<E: Fail> Fail for BodyError<E> {}

impl<E: HttpError> HttpError for BodyError<E> {
    fn status_code(&self) -> StatusCode {
        match self {
            BodyError::Parse { .. } => StatusCode::BAD_REQUEST,
            BodyError::Receiving { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Trait representing the transformation from a message body.
pub trait FromBody: 'static + Sized {
    /// The error type which will be returned from `from_data`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    #[allow(unused_variables)]
    fn is_match(input: &Input) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error>;
}

impl FromBody for Bytes {
    type Error = Never;

    fn from_body(body: Bytes, _: &Input) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = BadRequest;

    fn from_body(body: Bytes, _: &Input) -> Result<Self, Self::Error> {
        String::from_utf8(body.to_vec())
            .map_err(|_| BadRequest::new("failed to parse the message body"))
    }
}
