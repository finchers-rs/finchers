//! Components for parsing the HTTP headers.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use failure::Fail;
use futures_util::future;
use http::header::HeaderValue;
use http::StatusCode;

use endpoint::{Endpoint, EndpointExt};
use error::{Error, HttpError};
use generic::{one, One};
use input::{with_get_cx, Cursor, FromHeaderValue, Input};

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::endpoints::header;
/// # use finchers_core::local;
/// #
/// let endpoint = header::parse::<String>("x-api-key");
///
/// assert_eq!(
///     local::get("/")
///         .header("x-api-key", "some-api-key")
///         .apply(&endpoint)
///         .map(|res| res.map_err(drop)),
///     Some(Ok(("some-api-key".into(),)))
/// );
///
/// assert_eq!(
///     local::get("/")
///         .apply(&endpoint)
///         .map(|res| res.map_err(drop)),
///     None
/// );
/// ```
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # #![feature(use_extern_macros)]
/// #
/// # extern crate finchers_core;
/// # extern crate failure;
/// # extern crate http;
/// #
/// # use finchers_core::endpoint::{reject, EndpointExt};
/// # use finchers_core::endpoints::header;
/// # use finchers_core::error::HttpError;
/// # use finchers_core::local;
/// # use failure::Fail;
/// # use http::StatusCode;
/// #
/// #[derive(Debug, Fail)]
/// #[fail(display = "missing api key")]
/// struct MissingAPIKey { _priv: () }
///
/// impl HttpError for MissingAPIKey {
///     fn status_code(&self) -> StatusCode {
///         StatusCode::BAD_REQUEST
///     }
/// }
///
/// let endpoint = header::parse::<String>("x-api-key")
///     .or(reject(|_| MissingAPIKey { _priv: () }));
///
/// assert_eq!(
///     local::get("/")
///         .header("x-api-key", "xxxx-xxxx-xxxx")
///         .apply(&endpoint),
///     Some(Ok(("xxxx-xxxx-xxxx".into(),)))
/// );
///
/// assert_eq!(
///     local::get("/")
///         .apply(&endpoint)
///         .map(|res| res.map_err(|e| e.to_string())),
///     Some(Err("missing api key".into()))
/// );
/// ```
pub fn parse<H>(name: &'static str) -> ParseHeader<H>
where
    H: FromHeaderValue,
    H::Error: Fail,
{
    (ParseHeader {
        name,
        _marker: PhantomData,
    }).output::<One<H>>()
}

/// An instance of endpoint for extracting a header value.
pub struct ParseHeader<H> {
    name: &'static str,
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for ParseHeader<H> {}

impl<H> Clone for ParseHeader<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for ParseHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParseHeader")
            .field("name", &self.name)
            .finish()
    }
}

impl<H> Endpoint for ParseHeader<H>
where
    H: FromHeaderValue,
    H::Error: Fail,
{
    type Output = One<H>;
    type Future = ParseHeaderFuture<H>;

    fn apply<'c>(
        &self,
        input: PinMut<Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        if input.headers().contains_key(self.name) {
            Some((
                ParseHeaderFuture {
                    name: self.name,
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
pub struct ParseHeaderFuture<H> {
    name: &'static str,
    _marker: PhantomData<fn() -> H>,
}

impl<H> fmt::Debug for ParseHeaderFuture<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParseHeaderFuture")
            .field("name", &self.name)
            .finish()
    }
}

impl<H> Future for ParseHeaderFuture<H>
where
    H: FromHeaderValue,
    H::Error: Fail,
{
    type Output = Result<One<H>, Error>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            match input.request().headers().get(self.name) {
                Some(h) => H::from_header_value(h)
                    .map(one)
                    .map_err(|cause| HeaderError { cause }.into()),
                None => unreachable!(),
            }
        }))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "failed to parse a header value: {}", cause)]
pub struct HeaderError<E: Fail> {
    cause: E,
}

impl<E: Fail> HttpError for HeaderError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// Creates an endpoint which validates an entry of header value.
///
/// # Examples
///
/// ```
/// use finchers_core::endpoint::EndpointExt;
/// use finchers_core::endpoints::header;
///
/// let endpoint = header::exact("accept", "*/*");
/// ```
pub fn exact<V>(name: &'static str, value: V) -> ExactHeader<V>
where
    HeaderValue: PartialEq<V>,
{
    (ExactHeader { name, value }).output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ExactHeader<V> {
    name: &'static str,
    value: V,
}

impl<V> Endpoint for ExactHeader<V>
where
    HeaderValue: PartialEq<V>,
{
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        input: PinMut<Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        match input.headers().get(self.name) {
            Some(h) if *h == self.value => Some((future::ready(Ok(())), cursor)),
            _ => None,
        }
    }
}
