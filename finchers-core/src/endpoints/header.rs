//! Components for parsing the HTTP headers.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use failure::Fail;
use http::StatusCode;

use crate::endpoint::{EndpointBase, EndpointExt};
use crate::error::HttpError;
use crate::generic::{one, One};
use crate::input::{with_get_cx, Cursor, FromHeader, Input};

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```ignore
/// #![feature(rust_2018_preview)]
/// # use finchers_core::http::header::{header, FromHeader};
/// # use finchers_core::ext::{EndpointExt, EndpointResultExt, EndpointOptionExt};
/// # use std::string::FromUtf8Error;
/// #
/// pub struct APIKey(pub String);
///
/// impl FromHeader for APIKey {
///     type Error = FromUtf8Error;
///
///     const NAME: &'static str = "X-API-Key";
///
///     fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
///         String::from_utf8(s.to_owned()).map(APIKey)
///     }
/// }
///
/// # fn main() {
/// let api_key = header::<APIKey>().unwrap_ok();
/// # }
/// ```
pub fn header<H>() -> Header<H>
where
    H: FromHeader,
    H::Error: Fail,
{
    (Header {
        _marker: PhantomData,
    }).ok::<One<H>>()
    .err::<HeaderError<H::Error>>()
}

#[allow(missing_docs)]
pub struct Header<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for Header<H> {}

impl<H> Clone for Header<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for Header<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Header").finish()
    }
}

impl<H> EndpointBase for Header<H>
where
    H: FromHeader,
    H::Error: Fail,
{
    type Ok = One<H>;
    type Error = HeaderError<H::Error>;
    type Future = HeaderFuture<H>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        if !H::ALLOW_SKIP || input.headers().contains_key(H::NAME) {
            Some((
                HeaderFuture {
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
pub struct HeaderFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Future for HeaderFuture<H>
where
    H: FromHeader,
    H::Error: Fail,
{
    type Output = Result<One<H>, HeaderError<H::Error>>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        Poll::Ready(
            with_get_cx(|input| match input.request().headers().get(H::NAME) {
                Some(h) => H::from_header(h.as_bytes())
                    .map_err(|cause| HeaderError::InvalidValue { cause }),
                None => H::default().ok_or_else(|| HeaderError::MissingValue),
            }).map(one),
        )
    }
}

/// All kinds of error which will be returned from `Header<H>`.
#[derive(Debug)]
pub enum HeaderError<E> {
    /// The required header value was missing in the incoming request.
    MissingValue,

    /// Failed to parse the header value to a given type.
    #[allow(missing_docs)]
    InvalidValue { cause: E },
}

impl<E: fmt::Display> fmt::Display for HeaderError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeaderError::MissingValue => formatter.write_str("missing header value"),
            HeaderError::InvalidValue { ref cause } => {
                write!(formatter, "failed to parse a header value: {}", cause)
            }
        }
    }
}

impl<E: Fail> Fail for HeaderError<E> {}

impl<E: Fail> HttpError for HeaderError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
