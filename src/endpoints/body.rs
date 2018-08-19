//! Endpoints for parsing the message body.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, mem, task};

use bytes::Bytes;
use bytes::BytesMut;
use futures_util::try_ready;
use http::StatusCode;
use mime;
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use serde::de::DeserializeOwned;
use serde_json;

use crate::endpoint::{Context, Endpoint, EndpointExt, EndpointResult};
use crate::error::{bad_request, err_msg, Error};
use crate::generic::{one, One};
use crate::input::body::{FromBody, Payload};
use crate::input::query::{FromQuery, QueryItems};
use crate::input::with_get_cx;

/// Creates an endpoint which takes the instance of [`Payload`](input::body::Payload)
/// from the context.
///
/// If the instance of `Payload` has already been stolen by another endpoint, it will
/// return an error.
pub fn raw() -> Raw {
    (Raw { _priv: () }).output::<One<Payload>>()
}

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct Raw {
    _priv: (),
}

impl fmt::Debug for Raw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Raw").finish()
    }
}

impl Endpoint for Raw {
    type Output = One<Payload>;
    type Future = RawFuture;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(RawFuture { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawFuture {
    _priv: (),
}

impl Future for RawFuture {
    type Output = Result<One<Payload>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(
            with_get_cx(|input| input.payload())
                .map(one)
                .ok_or_else(stolen_payload),
        )
    }
}

#[derive(Debug)]
struct ReceiveAll {
    state: State,
}

#[derive(Debug)]
enum State {
    Start,
    Receiving(Payload, BytesMut),
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
                        None => return Poll::Ready(Err(stolen_payload())),
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

fn stolen_payload() -> Error {
    err_msg(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The instance of Payload has already been stolen by another endpoint.",
    )
}

// ==== Body ====

/// Creates an endpoint which receives the all contents of the message body
/// and transform the received bytes into a value of `T`.
pub fn parse<T>() -> Parse<T>
where
    T: FromBody,
{
    (Parse {
        _marker: PhantomData,
    }).output::<One<T>>()
}

#[allow(missing_docs)]
pub struct Parse<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Parse<T> {}

impl<T> Clone for Parse<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse").finish()
    }
}

impl<T> Endpoint for Parse<T>
where
    T: FromBody,
{
    type Output = One<T>;
    type Future = ParseFuture<T>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(ParseFuture {
            receive_all: ReceiveAll::new(),
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ParseFuture<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ParseFuture<T> {
    unsafe_pinned!(receive_all: ReceiveAll);
}

impl<T> Future for ParseFuture<T>
where
    T: FromBody,
{
    type Output = Result<One<T>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let data = try_ready!(self.receive_all().poll(cx));
        Poll::Ready(
            with_get_cx(|input| T::from_body(data, input))
                .map(one)
                .map_err(bad_request),
        )
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

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(JsonFuture {
            receive_all: ReceiveAll::new(),
            _marker: PhantomData,
        })
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
            Ok(Some(m)) if *m != mime::APPLICATION_JSON => {
                Some(bad_request("The content type must be application/json"))
            }
            Err(err) => Some(bad_request(err)),
            _ => None,
        });
        if let Some(err) = err {
            return Poll::Ready(Err(err));
        }

        let data = try_ready!(self.receive_all().poll(cx));
        Poll::Ready(serde_json::from_slice(&*data).map(one).map_err(bad_request))
    }
}

// ==== Form ====

/// Create an endpoint which parses an urlencoded data.
pub fn urlencoded<T>() -> UrlEncoded<T>
where
    T: FromQuery,
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
{
    type Output = (T,);
    type Future = UrlEncodedFuture<T>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(UrlEncodedFuture {
            receive_all: ReceiveAll::new(),
            _marker: PhantomData,
        })
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
{
    type Output = Result<(T,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let err = with_get_cx(|input| match input.content_type() {
            Ok(Some(m)) if *m != mime::APPLICATION_WWW_FORM_URLENCODED => Some(bad_request(
                "The content type must be application/www-x-urlformencoded",
            )),
            Err(err) => Some(bad_request(err)),
            _ => None,
        });
        if let Some(err) = err {
            return Poll::Ready(Err(err));
        }

        let data = try_ready!(self.receive_all().poll(cx));
        let items = unsafe { QueryItems::new_unchecked(&*data) };
        Poll::Ready(FromQuery::from_query(items).map(one).map_err(bad_request))
    }
}
