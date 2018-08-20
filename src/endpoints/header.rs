//! Components for parsing the HTTP headers.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use failure::format_err;
use futures_util::future;
use http::header::HeaderValue;

use crate::endpoint::{Context, Endpoint, EndpointErrorKind, EndpointExt, EndpointResult};
use crate::error::{bad_request, Error};
use crate::generic::{one, One};
use crate::input::header::FromHeader;
use crate::input::with_get_cx;

// ==== Required ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoints will skip the request if the specified header is missing.
///
/// # Example
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # extern crate finchers;
/// # extern crate http;
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::endpoints::header;
/// # use finchers::input::header::FromHeader;
/// # use finchers::rt::local;
/// # use http::header::HeaderValue;
/// #
/// #[derive(Debug, PartialEq)]
/// struct APIKey(String);
///
/// impl FromHeader for APIKey {
///     // ...
/// #    const HEADER_NAME: &'static str = "x-api-key";
/// #    type Error = std::str::Utf8Error;
/// #    fn from_header(value: &HeaderValue) -> Result<Self, Self::Error> {
/// #        std::str::from_utf8(value.as_bytes())
/// #            .map(ToOwned::to_owned)
/// #            .map(APIKey)
/// #    }
/// }
///
/// let endpoint = header::required::<APIKey>();
///
/// assert_eq!(
///     local::get("/")
///         .header("x-api-key", "some-api-key")
///         .apply(&endpoint)
///         .map_err(drop),
///     Ok((APIKey("some-api-key".into()),))
/// );
/// ```
pub fn required<H>() -> Required<H>
where
    H: FromHeader + 'static,
{
    (Required {
        _marker: PhantomData,
    }).output::<One<H>>()
}

#[allow(missing_docs)]
pub struct Required<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for Required<H> {}

impl<H> Clone for Required<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for Required<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Required").finish()
    }
}

impl<'e, H> Endpoint<'e> for Required<H>
where
    H: FromHeader + 'static,
{
    type Output = One<H>;
    type Future = RequiredFuture<H>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        if ecx.input().headers().contains_key(H::HEADER_NAME) {
            Ok(RequiredFuture {
                _marker: PhantomData,
            })
        } else {
            Err(EndpointErrorKind::Other(bad_request(format_err!(
                "missing header: `{}'",
                H::HEADER_NAME
            ))))
        }
    }
}

#[doc(hidden)]
pub struct RequiredFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> fmt::Debug for RequiredFuture<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequiredFuture").finish()
    }
}

impl<H> Future for RequiredFuture<H>
where
    H: FromHeader,
{
    type Output = Result<One<H>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            match input.request().headers().get(H::HEADER_NAME) {
                Some(h) => H::from_header(h).map(one).map_err(bad_request),
                None => unreachable!(),
            }
        }))
    }
}

// ==== Optional ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint will not skip the request and will return an error if the
/// header value is missing.
///
/// # Example
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # extern crate finchers;
/// # extern crate http;
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::endpoints::header;
/// # use finchers::input::header::FromHeader;
/// # use finchers::rt::local;
/// # use http::StatusCode;
/// # use http::header::HeaderValue;
/// #
/// #[derive(Debug, PartialEq)]
/// struct APIKey(String);
/// impl FromHeader for APIKey {
///     // ...
/// #    const HEADER_NAME: &'static str = "x-api-key";
/// #    type Error = std::str::Utf8Error;
/// #    fn from_header(value: &HeaderValue) -> Result<Self, Self::Error> {
/// #        std::str::from_utf8(value.as_bytes())
/// #            .map(ToOwned::to_owned)
/// #            .map(APIKey)
/// #    }
/// }
///
/// let endpoint = header::optional::<APIKey>();
///
/// assert_eq!(
///     local::get("/")
///         .header("x-api-key", "some-api-key")
///         .apply(&endpoint)
///         .map_err(drop),
///     Ok((Some(APIKey("some-api-key".into())),))
/// );
/// ```
pub fn optional<H>() -> Optional<H>
where
    H: FromHeader + 'static,
{
    (Optional {
        _marker: PhantomData,
    }).output::<One<Option<H>>>()
}

#[allow(missing_docs)]
pub struct Optional<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for Optional<H> {}

impl<H> Clone for Optional<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for Optional<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Optional").finish()
    }
}

impl<'e, H> Endpoint<'e> for Optional<H>
where
    H: FromHeader + 'static,
{
    type Output = One<Option<H>>;
    type Future = OptionalFuture<H>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(OptionalFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
pub struct OptionalFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> fmt::Debug for OptionalFuture<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseHeaderFuture").finish()
    }
}

impl<H> Future for OptionalFuture<H>
where
    H: FromHeader,
{
    type Output = Result<One<Option<H>>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            match input.request().headers().get(H::HEADER_NAME) {
                Some(h) => H::from_header(h).map(|h| one(Some(h))).map_err(bad_request),
                None => Ok(one(None)),
            }
        }))
    }
}

// ==== Exact ====

/// Creates an endpoint which validates an entry of header value.
///
/// # Examples
///
/// ```
/// use finchers::endpoint::EndpointExt;
/// use finchers::endpoints::header;
///
/// let endpoint = header::exact("accept", "*/*");
/// ```
pub fn exact<V>(name: &'static str, value: V) -> Exact<V>
where
    HeaderValue: PartialEq<V>,
{
    (Exact { name, value }).output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Exact<V> {
    name: &'static str,
    value: V,
}

impl<'e, V> Endpoint<'e> for Exact<V>
where
    HeaderValue: PartialEq<V>,
    V: 'e,
{
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match ecx.input().headers().get(self.name) {
            Some(h) if *h == self.value => Ok(future::ready(Ok(()))),
            _ => Err(EndpointErrorKind::NotMatched),
        }
    }
}
