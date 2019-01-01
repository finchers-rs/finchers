//! Endpoints for parsing the message body.

use std::mem;

use bytes::{Bytes, BytesMut};
use http::StatusCode;
use hyper::body::{Body, Payload as _Payload};
use serde::de::DeserializeOwned;

use crate::endpoint::{ApplyError, Endpoint};
use crate::error;
use crate::error::{err_msg, Error};
use crate::future::EndpointFuture;

/// Creates an endpoint which takes the instance of [`Payload`]
/// from the context.
///
/// If the instance of `Payload` has already been stolen by another endpoint, it will
/// return an error.
///
/// [`Payload`]: ../input/struct.Payload.html
#[inline]
pub fn raw() -> impl Endpoint<
    Output = (hyper::Body,),
    Future = impl EndpointFuture<Output = (hyper::Body,)> + Send + 'static, //
> {
    crate::endpoint::apply_fn(|_| {
        Ok(crate::future::poll_fn(|cx| {
            cx.body()
                .take()
                .map(|x| (x,).into())
                .ok_or_else(stolen_payload)
        }))
    })
}

/// Creates an endpoint which receives all of request body.
///
/// If the instance of `Payload` has already been stolen by another endpoint, it will
/// return an error.
#[inline]
pub fn receive_all() -> impl Endpoint<
    Output = (Bytes,),
    Future = impl EndpointFuture<Output = (Bytes,)> + Send + 'static, //
> {
    crate::endpoint::apply_fn(|_| Ok(receive_all_future()))
}

// ==== Text ====

/// Create an endpoint which parses a request body into `String`.
#[inline]
pub fn text() -> impl Endpoint<
    Output = (String,),
    Future = impl EndpointFuture<Output = (String,)> + Send + 'static, //
> {
    crate::endpoint::apply_fn(|cx| {
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

        let mut receive_all = receive_all_future();
        Ok(crate::future::poll_fn(move |cx| {
            let (data,) = futures::try_ready!(receive_all.poll_endpoint(cx));
            String::from_utf8(data.to_vec())
                .map(|x| (x,).into())
                .map_err(crate::error::bad_request)
        }))
    })
}

/// Create an endpoint which parses a request body into a JSON data.
#[inline]
pub fn json<T>() -> impl Endpoint<
    Output = (T,),
    Future = impl EndpointFuture<Output = (T,)> + Send + 'static, //
>
where
    T: DeserializeOwned + 'static,
{
    crate::endpoint::apply_fn(|cx| {
        let content_type = cx.content_type().map_err(ApplyError::custom)?;
        let m = content_type
            .ok_or_else(|| ApplyError::custom(crate::error::bad_request("missing content type")))?;
        if *m != mime::APPLICATION_JSON {
            return Err(ApplyError::custom(crate::error::bad_request(
                "The value of `Content-type` must be `application/json`.",
            )));
        }

        let mut receive_all = receive_all_future();
        Ok(crate::future::poll_fn(move |cx| {
            let (data,) = futures::try_ready!(receive_all.poll_endpoint(cx));
            serde_json::from_slice(&*data)
                .map(|x| (x,).into())
                .map_err(crate::error::bad_request)
        }))
    })
}

// ==== UrlEncoded ====

/// Create an endpoint which parses an urlencoded data.
#[inline]
pub fn urlencoded<T>() -> impl Endpoint<
    Output = (T,),
    Future = impl EndpointFuture<Output = (T,)> + Send + 'static, //
>
where
    T: DeserializeOwned + 'static,
{
    crate::endpoint::apply_fn(|cx| {
        let content_type = cx.content_type().map_err(ApplyError::custom)?;
        let m = content_type
            .ok_or_else(|| ApplyError::custom(crate::error::bad_request("missing content type")))?;
        if *m != mime::APPLICATION_WWW_FORM_URLENCODED {
            return Err(ApplyError::custom(crate::error::bad_request(
                "The value of `Content-type` must be `application-x-www-form-urlencoded`.",
            )));
        }

        let mut receive_all = receive_all_future();
        Ok(crate::future::poll_fn(move |cx| {
            let (data,) = futures::try_ready!(receive_all.poll_endpoint(cx));
            let s = std::str::from_utf8(&*data).map_err(crate::error::bad_request)?;
            serde_qs::from_str(s)
                .map(|x| (x,).into())
                .map_err(|err| crate::error::bad_request(failure::SyncFailure::new(err)))
        }))
    })
}

// ==== Util ====

fn stolen_payload() -> Error {
    err_msg(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The instance of Payload has already been stolen by another endpoint.",
    )
}

fn receive_all_future() -> impl EndpointFuture<Output = (Bytes,)> {
    #[allow(missing_debug_implementations)]
    enum State {
        Start,
        Receiving(Body, BytesMut),
        Done,
    }

    let mut state = State::Start;

    crate::future::poll_fn(move |cx| loop {
        state = match state {
            State::Start => {
                let payload = cx.body().take().ok_or_else(stolen_payload)?;
                State::Receiving(payload, BytesMut::new())
            }
            State::Receiving(ref mut body, ref mut buf) => {
                while let Some(data) = futures::try_ready!(body.poll_data().map_err(error::fail)) {
                    buf.extend_from_slice(&*data);
                }
                let buf = mem::replace(buf, BytesMut::new()).freeze();
                return Ok::<_, crate::error::Error>((buf,).into());
            }
            _ => panic!("cannot resolve/reject twice"),
        };
    })
}
