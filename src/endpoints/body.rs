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
use input::{with_get_cx, Cursor, FromBody, Input, RequestBody};

/// Creates an endpoint which will take the instance of `RequestBody` from the context.
///
/// If the instance has already been stolen by another Future, this endpoint will return
/// a `None`.
pub fn raw_body() -> RawBody {
    (RawBody { _priv: () }).output::<One<RequestBody>>()
}

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct RawBody {
    _priv: (),
}

impl fmt::Debug for RawBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawBody").finish()
    }
}

impl Endpoint for RawBody {
    type Output = One<RequestBody>;
    type Future = RawBodyFuture;

    fn apply(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((RawBodyFuture { _priv: () }, cursor))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawBodyFuture {
    _priv: (),
}

impl Future for RawBodyFuture {
    type Output = Result<One<RequestBody>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        let body = with_get_cx(|input| input.body());
        Poll::Ready(Ok(one(body)))
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
    Receiving(RequestBody, BytesMut),
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

    fn poll(mut self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        'poll: loop {
            match self.state() {
                State::Init => {}
                State::Receiving(ref mut body, ref mut buf) => {
                    while let Some(data) = try_ready!(body.poll_data()) {
                        buf.extend_from_slice(&*data);
                    }
                }
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
