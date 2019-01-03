//! Endpoints for parsing the message body.

use {
    crate::{
        endpoint::{ApplyContext, ApplyError, ApplyResult, Endpoint},
        error::Error,
        future::{Context, EndpointFuture, Poll},
    },
    http::StatusCode,
    izanami_service::http::BufStream,
    serde::de::DeserializeOwned,
    std::{cell::UnsafeCell, marker::PhantomData},
};

fn stolen_payload() -> Error {
    crate::error::err_msg(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The instance of BufStream has already been stolen by another endpoint.",
    )
}

/// Creates an endpoint which takes the instance of request body from the context.
///
/// If the instance of request body has already been stolen by another endpoint,
/// it will return an error.
#[inline]
pub fn raw() -> Raw {
    Raw(())
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Raw(());

mod raw {
    use super::*;

    impl<Bd> Endpoint<Bd> for Raw {
        type Output = (Bd,);
        type Future = RawFuture;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            Ok(RawFuture {
                _anchor: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RawFuture {
        _anchor: PhantomData<UnsafeCell<()>>,
    }

    impl<Bd> EndpointFuture<Bd> for RawFuture {
        type Output = (Bd,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            cx.body()
                .take()
                .map(|x| (x,).into())
                .ok_or_else(stolen_payload)
        }
    }
}

/// Creates an endpoint which receives all of request body.
///
/// If the instance of `BufStream` has already been stolen by another endpoint, it will
/// return an error.
#[inline]
pub fn receive_all() -> ReceiveAll {
    ReceiveAll(())
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ReceiveAll(());

mod receive_all {
    use super::*;
    use bytes::Buf;

    impl<Bd> Endpoint<Bd> for ReceiveAll
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        type Output = (Vec<u8>,);
        type Future = ReceiveAllFuture<Bd>;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            Ok(future())
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct ReceiveAllFuture<Bd> {
        state: State<Bd>,
    }

    #[allow(missing_debug_implementations)]
    enum State<Bd> {
        Start,
        Receiving(Bd, Vec<u8>),
    }

    impl<Bd> EndpointFuture<Bd> for ReceiveAllFuture<Bd>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        type Output = (Vec<u8>,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            loop {
                self.state = match self.state {
                    State::Start => {
                        let payload = cx.body().take().ok_or_else(super::stolen_payload)?;
                        State::Receiving(payload, Vec::new())
                    }
                    State::Receiving(ref mut body, ref mut buf) => {
                        while let Some(data) = futures::try_ready!(body
                            .poll_buf()
                            .map_err(|e| failure::Error::from_boxed_compat(e.into())))
                        {
                            buf.extend_from_slice(data.bytes());
                        }
                        let buf = std::mem::replace(buf, Vec::new());
                        return Ok((buf,).into());
                    }
                };
            }
        }
    }

    pub(super) fn future<Bd>() -> ReceiveAllFuture<Bd>
    where
        Bd: BufStream,
    {
        ReceiveAllFuture {
            state: State::Start,
        }
    }
}

// ==== Text ====

/// Create an endpoint which parses a request body into `String`.
#[inline]
pub fn text() -> Text {
    Text {
        receive_all: receive_all(),
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Text {
    receive_all: ReceiveAll,
}

mod text {
    use super::*;

    impl<Bd> Endpoint<Bd> for Text
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        type Output = (String,);
        type Future = TextFuture<Bd>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            let content_type = cx.content_type().map_err(ApplyError::custom)?;
            match content_type.and_then(|m| m.get_param("charset")) {
                Some(ref val) if *val == "utf-8" => {}
                Some(_val) => {
                    return Err(ApplyError::custom(crate::error::bad_request(
                        "Only the UTF-8 charset is supported.",
                    )))
                }
                None => {}
            }

            Ok(TextFuture {
                receive_all: self.receive_all.apply(cx)?,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct TextFuture<Bd>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        receive_all: <ReceiveAll as Endpoint<Bd>>::Future,
    }

    impl<Bd> EndpointFuture<Bd> for TextFuture<Bd>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        type Output = (String,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            let (data,) = futures::try_ready!(self.receive_all.poll_endpoint(cx));
            String::from_utf8(data.to_vec())
                .map(|x| (x,).into())
                .map_err(crate::error::bad_request)
        }
    }
}

/// Create an endpoint which parses a request body into a JSON data.
#[inline]
pub fn json<T>() -> Json<T>
where
    T: DeserializeOwned,
{
    Json {
        receive_all: receive_all(),
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Json<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

mod json {
    use super::*;
    use std::fmt;

    impl<T> fmt::Debug for Json<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Json").finish()
        }
    }

    impl<T, Bd> Endpoint<Bd> for Json<T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
        T: DeserializeOwned,
    {
        type Output = (T,);
        type Future = JsonFuture<Bd, T>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            let content_type = cx.content_type().map_err(ApplyError::custom)?;
            let m = content_type.ok_or_else(|| {
                ApplyError::custom(crate::error::bad_request("missing content type"))
            })?;
            if *m != mime::APPLICATION_JSON {
                return Err(ApplyError::custom(crate::error::bad_request(
                    "The value of `Content-type` must be `application/json`.",
                )));
            }

            Ok(JsonFuture {
                receive_all: self.receive_all.apply(cx)?,
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct JsonFuture<Bd, T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        receive_all: <ReceiveAll as Endpoint<Bd>>::Future,
        _marker: PhantomData<fn() -> T>,
    }

    impl<Bd, T> EndpointFuture<Bd> for JsonFuture<Bd, T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
        T: DeserializeOwned,
    {
        type Output = (T,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            let (data,) = futures::try_ready!(self.receive_all.poll_endpoint(cx));
            serde_json::from_slice(&*data)
                .map(|x| (x,).into())
                .map_err(crate::error::bad_request)
        }
    }
}

// ==== UrlEncoded ====

/// Create an endpoint which parses an urlencoded data.
#[inline]
pub fn urlencoded<T>() -> Urlencoded<T>
where
    T: DeserializeOwned,
{
    Urlencoded {
        receive_all: receive_all(),
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Urlencoded<T> {
    receive_all: ReceiveAll,
    _marker: PhantomData<fn() -> T>,
}

mod urlencoded {
    use super::*;
    use {failure::SyncFailure, std::fmt};

    impl<T> fmt::Debug for Urlencoded<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Json").finish()
        }
    }

    impl<T, Bd> Endpoint<Bd> for Urlencoded<T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
        T: DeserializeOwned,
    {
        type Output = (T,);
        type Future = UrlencodedFuture<Bd, T>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            let content_type = cx.content_type().map_err(ApplyError::custom)?;
            let m = content_type.ok_or_else(|| {
                ApplyError::custom(crate::error::bad_request("missing content type"))
            })?;
            if *m != mime::APPLICATION_WWW_FORM_URLENCODED {
                return Err(ApplyError::custom(crate::error::bad_request(
                    "The value of `Content-type` must be `application-x-www-form-urlencoded`.",
                )));
            }

            Ok(UrlencodedFuture {
                receive_all: self.receive_all.apply(cx)?,
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct UrlencodedFuture<Bd, T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        receive_all: <ReceiveAll as Endpoint<Bd>>::Future,
        _marker: PhantomData<fn() -> T>,
    }

    impl<Bd, T> EndpointFuture<Bd> for UrlencodedFuture<Bd, T>
    where
        Bd: BufStream,
        Bd::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
        T: DeserializeOwned,
    {
        type Output = (T,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            let (data,) = futures::try_ready!(self.receive_all.poll_endpoint(cx));
            let s = std::str::from_utf8(&*data).map_err(crate::error::bad_request)?;
            serde_qs::from_str(s)
                .map(|x| (x,).into())
                .map_err(|err| crate::error::bad_request(SyncFailure::new(err)))
        }
    }
}
