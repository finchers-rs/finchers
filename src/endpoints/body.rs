//! Components for parsing the HTTP request body.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, mem, task};

use bytes::BytesMut;
use failure::Fail;
use futures_util::try_ready;
use http::StatusCode;
use pin_utils::unsafe_unpinned;

use endpoint::{Endpoint, EndpointExt};
use error::{Error, HttpError};
use generic::{one, One};
use input::body::FromBody;
use input::{self, with_get_cx, Cursor, Input};

/// Creates an endpoint which will take the instance of `Payload` from the context.
///
/// If the instance has already been stolen by another Future, this endpoint will return
/// an error.
pub fn payload() -> Payload {
    (Payload { _priv: () }).output::<One<input::body::Payload>>()
}

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct Payload {
    _priv: (),
}

impl fmt::Debug for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Payload").finish()
    }
}

impl Endpoint for Payload {
    type Output = One<input::body::Payload>;
    type Future = PayloadFuture;

    fn apply(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((PayloadFuture { _priv: () }, cursor))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct PayloadFuture {
    _priv: (),
}

impl Future for PayloadFuture {
    type Output = Result<One<input::body::Payload>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        let payload = with_get_cx(|input| input.payload());
        Poll::Ready(payload.map(one).ok_or_else(|| StolenPayload.into()))
    }
}

/// Creates an endpoint which will poll the all contents of the message body
/// from the client and transform the received bytes into a value of `T`.
pub fn body<T>() -> Body<T>
where
    T: FromBody,
    T::Error: Fail,
{
    (Body {
        _marker: PhantomData,
    }).output::<One<T>>()
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T> Endpoint for Body<T>
where
    T: FromBody,
    T::Error: Fail,
{
    type Output = One<T>;
    type Future = BodyFuture<T>;

    fn apply(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        if T::is_match(input) {
            Some((BodyFuture { state: State::Init }, cursor))
        } else {
            None
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
    Receiving(input::body::Payload, BytesMut),
    Done,
    #[doc(hidden)]
    __NonExhausive(PhantomData<fn() -> T>),
}

impl<T> BodyFuture<T> {
    unsafe_unpinned!(state: State<T>);
}

impl<T> Future for BodyFuture<T>
where
    T: FromBody,
    T::Error: Fail,
{
    type Output = Result<One<T>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        'poll: loop {
            match self.state() {
                State::Init => {}
                State::Receiving(ref mut body, ref mut buf) => {
                    let mut body = unsafe { PinMut::new_unchecked(body) };
                    while let Some(data) = try_ready!(body.reborrow().poll_data(cx)) {
                        buf.extend_from_slice(&*data);
                    }
                }
                _ => panic!("cannot resolve/reject twice"),
            };

            match mem::replace(self.state(), State::Done) {
                State::Init => {
                    let payload = match with_get_cx(|input| input.payload()) {
                        Some(payload) => payload,
                        None => return Poll::Ready(Err(StolenPayload.into())),
                    };
                    *self.state() = State::Receiving(payload, BytesMut::new());
                    continue 'poll;
                }
                State::Receiving(_, buf) => {
                    return Poll::Ready(
                        with_get_cx(|input| T::from_body(buf.freeze(), input))
                            .map(one)
                            .map_err(|cause| BodyParseError { cause }.into()),
                    );
                }
                _ => panic!(),
            }
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "failed to parse the request body: {}", cause)]
pub struct BodyParseError<E: Fail> {
    cause: E,
}

impl<E: Fail> HttpError for BodyParseError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "The instance of Payload has already been stolen by another endpoint.")]
pub struct StolenPayload;

impl HttpError for StolenPayload {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
