//! Components for parsing the HTTP request body.

use std::future::Future;
use std::marker::PhantomData;
use std::marker::Unpin;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, mem, task};

use bytes::BytesMut;
use failure::Fail;
use futures_util::try_ready;
use http::StatusCode;
use pin_utils::unsafe_unpinned;

use crate::endpoint::{EndpointBase, EndpointExt};
use crate::error::{HttpError, Never};
use crate::generic::{one, One};
use crate::input::{with_get_cx, Cursor, FromBody, Input, PollDataError, RequestBody};

/// Creates an endpoint which will take the instance of `RequestBody` from the context.
///
/// If the instance has already been stolen by another Future, this endpoint will return
/// a `None`.
pub fn raw_body() -> RawBody {
    (RawBody { _priv: () })
        .ok::<One<RequestBody>>()
        .err::<Never>()
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
    type Ok = One<RequestBody>;
    type Error = Never;
    type Future = RawBodyFuture;

    fn apply(&self, _: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        Some((RawBodyFuture { _priv: () }, cursor))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawBodyFuture {
    _priv: (),
}

impl Future for RawBodyFuture {
    type Output = Result<One<RequestBody>, Never>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        let body = with_get_cx(|input| input.body());
        Poll::Ready(Ok(one(body)))
    }
}

/// Creates an endpoint which will poll the all contents of the message body
/// from the client and transform the received bytes into a value of `T`.
pub fn body<T>() -> Body<T>
where
    T: FromBody,
{
    (Body {
        _marker: PhantomData,
    }).ok::<One<T>>()
    .err::<BodyError<T::Error>>()
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
    type Ok = One<T>;
    type Error = BodyError<T::Error>;
    type Future = BodyFuture<T>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        match T::is_match(input) {
            true => Some((BodyFuture { state: State::Init }, cursor)),
            false => None,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct BodyFuture<T> {
    state: State<T>,
}

#[allow(missing_debug_implementations)]
enum State<T> {
    Init,
    Receiving(RequestBody, BytesMut),
    Done,
    #[doc(hidden)]
    __NonExhausive(PhantomData<fn() -> T>),
}

impl<T> BodyFuture<T> {
    unsafe_unpinned!(state: State<T>);
}

impl<T> Unpin for BodyFuture<T> {}

impl<T: FromBody> Future for BodyFuture<T> {
    type Output = Result<One<T>, BodyError<T::Error>>;

    fn poll(mut self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        'poll: loop {
            match self.state() {
                State::Init => {}
                State::Receiving(ref mut body, ref mut buf) => while let Some(data) =
                    try_ready!(body.poll_data().map_err(BodyError::Receiving))
                {
                    buf.extend_from_slice(&*data);
                },
                _ => panic!("cannot resolve/reject twice"),
            };

            match mem::replace(self.state(), State::Done) {
                State::Init => {
                    let body = with_get_cx(|input| input.body());
                    *self.state() = State::Receiving(body, BytesMut::new());
                    continue 'poll;
                }
                State::Receiving(_, buf) => {
                    return Poll::Ready(
                        with_get_cx(|input| T::from_body(buf.freeze(), input))
                            .map(one)
                            .map_err(BodyError::Parse),
                    );
                }
                _ => panic!(),
            }
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
