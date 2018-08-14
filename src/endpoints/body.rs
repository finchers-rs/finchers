//! Components for parsing the HTTP request body.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, mem, task};

use bytes::Bytes;
use bytes::BytesMut;
use failure::{format_err, Fail};
use futures_util::try_ready;
use http::StatusCode;
use mime;
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use serde::de::DeserializeOwned;
use serde_json;

use endpoint::{Endpoint, EndpointExt};
use error::{Error, Failure, HttpError};
use generic::{one, One};
use input::body::FromBody;
use input::query::{FromQuery, QueryItems};
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

#[derive(Debug)]
struct ReceiveAll {
    state: State,
}

#[derive(Debug)]
enum State {
    Start,
    Receiving(input::body::Payload, BytesMut),
    Done,
}

impl ReceiveAll {
    unsafe_unpinned!(state: State);

    fn new() -> ReceiveAll {
        ReceiveAll {
            state: State::Start,
        }
    }
}

impl Future for ReceiveAll {
    type Output = Result<Bytes, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        'poll: loop {
            match self.state() {
                State::Start => {}
                State::Receiving(ref mut body, ref mut buf) => {
                    let mut body = unsafe { PinMut::new_unchecked(body) };
                    while let Some(data) = try_ready!(body.reborrow().poll_data(cx)) {
                        buf.extend_from_slice(&*data);
                    }
                }
                _ => panic!("cannot resolve/reject twice"),
            };

            match mem::replace(self.state(), State::Done) {
                State::Start => {
                    let payload = match with_get_cx(|input| input.payload()) {
                        Some(payload) => payload,
                        None => return Poll::Ready(Err(StolenPayload.into())),
                    };
                    *self.state() = State::Receiving(payload, BytesMut::new());
                    continue 'poll;
                }
                State::Receiving(_, buf) => {
                    return Poll::Ready(Ok(buf.freeze()));
                }
                _ => panic!(),
            }
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "The instance of Payload has already been stolen by another endpoint.")]
struct StolenPayload;

impl HttpError for StolenPayload {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

// ==== Body ====

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
            Some((
                BodyFuture {
                    receive_all: ReceiveAll::new(),
                    _marker: PhantomData,
                },
                cursor,
            ))
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct BodyFuture<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

impl<T> BodyFuture<T> {
    unsafe_pinned!(receive_all: ReceiveAll);
}

impl<T> Future for BodyFuture<T>
where
    T: FromBody,
    T::Error: Fail,
{
    type Output = Result<One<T>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let data = try_ready!(self.receive_all().poll(cx));
        Poll::Ready(
            with_get_cx(|input| T::from_body(data, input))
                .map(one)
                .map_err(|cause| BodyParseError { cause }.into()),
        )
    }
}

#[derive(Debug, Fail)]
#[fail(display = "failed to parse the request body: {}", cause)]
struct BodyParseError<E: Fail> {
    cause: E,
}

impl<E: Fail> HttpError for BodyParseError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

// ==== Json ====

/// Create an endpoint which parses a request body into a JSON data.
pub fn json<T>() -> Json<T>
where
    T: DeserializeOwned,
{
    Json {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Json<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Endpoint for Json<T>
where
    T: DeserializeOwned,
{
    type Output = (T,);
    type Future = JsonFuture<T>;

    fn apply(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((
            JsonFuture {
                receive_all: ReceiveAll::new(),
                _marker: PhantomData,
            },
            cursor,
        ))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct JsonFuture<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

impl<T> JsonFuture<T> {
    unsafe_pinned!(receive_all: ReceiveAll);
}

impl<T> Future for JsonFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<(T,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let err = with_get_cx(|input| match input.content_type() {
            Ok(Some(m)) if *m != mime::APPLICATION_JSON => Some(Failure::new(
                StatusCode::BAD_REQUEST,
                format_err!("The content type must be application/json"),
            )),
            Err(err) => Some(Failure::new(StatusCode::BAD_REQUEST, err)),
            _ => None,
        });
        if let Some(err) = err {
            return Poll::Ready(Err(err.into()));
        }

        let data = try_ready!(self.receive_all().poll(cx));
        Poll::Ready(
            serde_json::from_slice(&*data)
                .map(one)
                .map_err(|cause| Failure::new(StatusCode::BAD_REQUEST, cause).into()),
        )
    }
}

// ==== Form ====

/// Create an endpoint which parses an urlencoded data.
pub fn urlencoded<T>() -> UrlEncoded<T>
where
    T: FromQuery,
    T::Error: Fail,
{
    UrlEncoded {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct UrlEncoded<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Endpoint for UrlEncoded<T>
where
    T: FromQuery,
    T::Error: Fail,
{
    type Output = (T,);
    type Future = UrlEncodedFuture<T>;

    fn apply(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((
            UrlEncodedFuture {
                receive_all: ReceiveAll::new(),
                _marker: PhantomData,
            },
            cursor,
        ))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct UrlEncodedFuture<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

impl<T> UrlEncodedFuture<T> {
    unsafe_pinned!(receive_all: ReceiveAll);
}

impl<T> Future for UrlEncodedFuture<T>
where
    T: FromQuery,
    T::Error: Fail,
{
    type Output = Result<(T,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let err = with_get_cx(|input| match input.content_type() {
            Ok(Some(m)) if *m != mime::APPLICATION_WWW_FORM_URLENCODED => Some(Failure::new(
                StatusCode::BAD_REQUEST,
                format_err!("The content type must be application/www-x-urlformencoded"),
            )),
            Err(err) => Some(Failure::new(StatusCode::BAD_REQUEST, err)),
            _ => None,
        });
        if let Some(err) = err {
            return Poll::Ready(Err(err.into()));
        }

        let data = try_ready!(self.receive_all().poll(cx));
        Poll::Ready(
            FromQuery::from_query(QueryItems::new(&*data))
                .map(one)
                .map_err(|cause| Failure::new(StatusCode::BAD_REQUEST, cause).into()),
        )
    }
}
