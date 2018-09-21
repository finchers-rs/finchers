//! Endpoints for parsing the message body.

use std::marker::PhantomData;
use std::pin::PinMut;
use std::{fmt, mem};

use futures_core::future::Future;
use futures_core::task;
use futures_core::task::Poll;
use futures_util::try_future;
use futures_util::try_future::TryFutureExt;
use futures_util::try_ready;

use bytes::Bytes;
use bytes::BytesMut;
use http::StatusCode;
use pin_utils::unsafe_unpinned;
use serde::de::DeserializeOwned;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::{err_msg, Error};
use crate::input::body::Payload;
use crate::input::query::FromQuery;
use crate::input::with_get_cx;

/// Creates an endpoint which takes the instance of [`Payload`](input::body::Payload)
/// from the context.
///
/// If the instance of `Payload` has already been stolen by another endpoint, it will
/// return an error.
#[inline]
pub fn raw() -> Raw {
    (Raw { _priv: () }).with_output::<(Payload,)>()
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

impl<'e> Endpoint<'e> for Raw {
    type Output = (Payload,);
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
    type Output = Result<(Payload,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(
            with_get_cx(|input| input.payload())
                .map(|x| (x,))
                .ok_or_else(stolen_payload),
        )
    }
}

/// Creates an endpoint which receives all of request body.
///
/// If the instance of `Payload` has already been stolen by another endpoint, it will
/// return an error.
#[inline]
pub fn receive_all() -> ReceiveAll {
    (ReceiveAll { _priv: () }).with_output::<(Bytes,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ReceiveAll {
    _priv: (),
}

impl<'a> Endpoint<'a> for ReceiveAll {
    type Output = (Bytes,);
    type Future = ReceiveAllFuture;

    fn apply(&'a self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(ReceiveAllFuture::new())
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ReceiveAllFuture {
    state: State,
}

#[derive(Debug)]
enum State {
    Start,
    Receiving(Payload, BytesMut),
    Done,
}

impl ReceiveAllFuture {
    unsafe_unpinned!(state: State);

    fn new() -> ReceiveAllFuture {
        ReceiveAllFuture {
            state: State::Start,
        }
    }
}

impl Future for ReceiveAllFuture {
    type Output = Result<(Bytes,), Error>;

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
                    return Poll::Ready(Ok((buf.freeze(),)));
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

#[allow(deprecated)]
mod deprecated_parse {
    use std::fmt;
    use std::marker::PhantomData;
    use std::pin::PinMut;

    use futures_core::future::Future;
    use futures_core::task;
    use futures_core::task::Poll;
    use futures_util::try_ready;
    use pin_utils::unsafe_pinned;

    use crate::endpoint::{Context, Endpoint, EndpointResult};
    use crate::error::{bad_request, Error};
    use crate::input::body::FromBody;
    use crate::input::with_get_cx;

    use super::ReceiveAllFuture;

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.3",
        note = "This function is going to remove before releasing 0.12.0."
    )]
    pub fn parse<T>() -> Parse<T>
    where
        T: FromBody,
    {
        Parse {
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.3",
        note = "This function is going to remove before releasing 0.12.0."
    )]
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

    impl<'e, T> Endpoint<'e> for Parse<T>
    where
        T: FromBody,
    {
        type Output = (T,);
        type Future = ParseFuture<T>;

        fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
            Ok(ParseFuture {
                receive_all: ReceiveAllFuture::new(),
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    #[doc(hidden)]
    #[deprecated(
        since = "0.12.0-alpha.3",
        note = "This function is going to remove before releasing 0.12.0."
    )]
    pub struct ParseFuture<T> {
        receive_all: ReceiveAllFuture,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T> ParseFuture<T> {
        unsafe_pinned!(receive_all: ReceiveAllFuture);
    }

    impl<T> Future for ParseFuture<T>
    where
        T: FromBody,
    {
        type Output = Result<(T,), Error>;

        fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
            let (data,) = try_ready!(self.receive_all().poll(cx));
            Poll::Ready(
                with_get_cx(|input| T::from_body(data, PinMut::new(input)))
                    .map(|x| (x,))
                    .map_err(bad_request),
            )
        }
    }
}

#[doc(hidden)]
#[allow(deprecated)]
pub use self::deprecated_parse::{parse, Parse, ParseFuture};

// ==== Text ====

/// Create an endpoint which parses a request body into `String`.
#[inline]
pub fn text() -> Text {
    (Text { _priv: () }).with_output::<(String,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Text {
    _priv: (),
}

impl<'a> Endpoint<'a> for Text {
    type Output = (String,);
    type Future = parse::ParseFuture<String>;

    fn apply(&'a self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(parse::ParseFuture::new())
    }
}

// ==== Json ====

/// Create an endpoint which parses a request body into a JSON data.
#[inline]
pub fn json<T>() -> Json<T>
where
    T: DeserializeOwned + 'static,
{
    (Json {
        _marker: PhantomData,
    }).with_output::<(T,)>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Json<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<'e, T> Endpoint<'e> for Json<T>
where
    T: DeserializeOwned + 'static,
{
    type Output = (T,);
    #[allow(clippy::type_complexity)]
    type Future =
        try_future::MapOk<parse::ParseFuture<parse::Json<T>>, fn((parse::Json<T>,)) -> (T,)>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(parse::ParseFuture::new().map_ok((|(parse::Json(v),)| (v,)) as fn(_) -> _))
    }
}

// ==== UrlEncoded ====

/// Create an endpoint which parses an urlencoded data.
#[inline]
pub fn urlencoded<T>() -> UrlEncoded<T>
where
    T: FromQuery,
{
    (UrlEncoded {
        _marker: PhantomData,
    }).with_output::<(T,)>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct UrlEncoded<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<'e, T> Endpoint<'e> for UrlEncoded<T>
where
    T: FromQuery,
{
    type Output = (T,);
    #[allow(clippy::type_complexity)]
    type Future = try_future::MapOk<
        parse::ParseFuture<parse::UrlEncoded<T>>,
        fn((parse::UrlEncoded<T>,)) -> (T,),
    >;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(parse::ParseFuture::new().map_ok((|(parse::UrlEncoded(v),)| (v,)) as fn(_) -> _))
    }
}

mod parse {
    use std::fmt;
    use std::marker::PhantomData;
    use std::pin::PinMut;

    use futures_core::future::Future;
    use futures_core::task;
    use futures_core::task::Poll;
    use futures_util::try_ready;
    use pin_utils::unsafe_pinned;

    use bytes::Bytes;
    use mime::Mime;
    use serde::de::DeserializeOwned;
    use serde_json;

    use crate::error::{bad_request, Error};
    use crate::input::query::{FromQuery, QueryItems};
    use crate::input::with_get_cx;

    use super::ReceiveAllFuture;

    pub struct ParseFuture<T> {
        receive_all: ReceiveAllFuture,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T> fmt::Debug for ParseFuture<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ParseFuture")
                .field("receive_all", &self.receive_all)
                .finish()
        }
    }

    impl<T> ParseFuture<T> {
        pub(super) fn new() -> ParseFuture<T> {
            ParseFuture {
                receive_all: ReceiveAllFuture::new(),
                _marker: PhantomData,
            }
        }

        unsafe_pinned!(receive_all: ReceiveAllFuture);
    }

    impl<T: FromBody> Future for ParseFuture<T> {
        type Output = Result<(T,), Error>;

        fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
            try_ready!(Poll::Ready(with_get_cx(|input| {
                let content_type = input.content_type().map_err(bad_request)?;
                T::validate(content_type)
            })));
            let (data,) = try_ready!(self.receive_all().poll(cx));
            Poll::Ready(T::parse(data).map(|x| (x,)))
        }
    }

    pub trait FromBody: Sized {
        fn validate(content_type: Option<&Mime>) -> Result<(), Error>;
        fn parse(body: Bytes) -> Result<Self, Error>;
    }

    impl FromBody for String {
        fn validate(content_type: Option<&Mime>) -> Result<(), Error> {
            match content_type.and_then(|m| m.get_param("charset")) {
                Some(ref val) if *val == "utf-8" => Ok(()),
                Some(_val) => Err(bad_request("Only the UTF-8 charset is supported.")),
                None => Ok(()),
            }
        }

        fn parse(body: Bytes) -> Result<Self, Error> {
            String::from_utf8(body.to_vec()).map_err(bad_request)
        }
    }

    #[derive(Debug)]
    pub struct Json<T>(pub T);

    impl<T: DeserializeOwned> FromBody for Json<T> {
        fn validate(content_type: Option<&Mime>) -> Result<(), Error> {
            let m = content_type.ok_or_else(|| bad_request("missing content type"))?;
            if *m != mime::APPLICATION_JSON {
                return Err(bad_request(
                    "The value of `Content-type` must be `application/json`.",
                ));
            }
            Ok(())
        }

        fn parse(body: Bytes) -> Result<Self, Error> {
            serde_json::from_slice(&*body)
                .map(Json)
                .map_err(bad_request)
        }
    }

    #[derive(Debug)]
    pub struct UrlEncoded<T>(pub T);

    impl<T: FromQuery> FromBody for UrlEncoded<T> {
        fn validate(content_type: Option<&Mime>) -> Result<(), Error> {
            let m = content_type.ok_or_else(|| bad_request("missing content type"))?;
            if *m != mime::APPLICATION_WWW_FORM_URLENCODED {
                return Err(bad_request(
                    "The value of `Content-type` must be `application-x-www-form-urlencoded`.",
                ));
            }
            Ok(())
        }

        fn parse(body: Bytes) -> Result<Self, Error> {
            let s = std::str::from_utf8(&*body).map_err(bad_request)?;
            let items = unsafe { QueryItems::new_unchecked(s) };
            FromQuery::from_query(items)
                .map(UrlEncoded)
                .map_err(bad_request)
        }
    }
}
