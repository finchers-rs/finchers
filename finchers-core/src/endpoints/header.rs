//! Components for parsing the HTTP headers.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use futures_util::future;
use http::header::HeaderValue;

use crate::endpoint::{EndpointBase, EndpointExt};
use crate::error::Never;
use crate::generic::{one, One};
use crate::input::{with_get_cx, Cursor, FromHeaderValue, Input};

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```
/// #![feature(rust_2018_preview)]
/// #
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::endpoints::header;
/// # use finchers_core::local;
///
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
/// #![feature(rust_2018_preview)]
/// #
/// # use finchers_core::endpoint::{reject, EndpointExt};
/// # use finchers_core::endpoints::header;
/// # use finchers_core::local;
/// # use failure::Fail;
///
/// #[derive(Debug, Fail)]
/// #[fail(display = "missing api key")]
/// struct MissingAPIKey { _priv: () }
///
/// let endpoint = header::parse::<String>("x-api-key")
///     .or(reject(|_| MissingAPIKey { _priv: () }));
///
/// assert_eq!(
///     local::get("/")
///         .header("x-api-key", "xxxx-xxxx-xxxx")
///         .apply(&endpoint)
///         .map(|res| res.map_err(drop)),
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
{
    (ParseHeader {
        name,
        _marker: PhantomData,
    }).ok::<One<H>>()
    .err::<H::Error>()
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

impl<H> EndpointBase for ParseHeader<H>
where
    H: FromHeaderValue,
{
    type Ok = One<H>;
    type Error = H::Error;
    type Future = ParseHeaderFuture<H>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
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
{
    type Output = Result<One<H>, H::Error>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            match input.request().headers().get(self.name) {
                Some(h) => H::from_header_value(h).map(one),
                None => unreachable!(),
            }
        }))
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
    (ExactHeader { name, value }).ok::<()>().err::<Never>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ExactHeader<V> {
    name: &'static str,
    value: V,
}

impl<V> EndpointBase for ExactHeader<V>
where
    HeaderValue: PartialEq<V>,
{
    type Ok = ();
    type Error = Never;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        match input.headers().get(self.name) {
            Some(h) if *h == self.value => Some((future::ready(Ok(())), cursor)),
            _ => None,
        }
    }
}
