//! Components for parsing the HTTP headers.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use failure::Fail;
use http::StatusCode;

use crate::endpoint::{Context, EndpointBase, EndpointExt};
use crate::error::HttpError;
use crate::generic::{one, One};
use crate::input::with_get_cx;

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```
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

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        if H::ALLOW_SKIP && !cx.input().request().headers().contains_key(H::NAME) {
            return None;
        }
        Some(HeaderFuture {
            _marker: PhantomData,
        })
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

/// Trait representing the conversion from an entry of HTTP header.
pub trait FromHeader: Sized {
    /// The error type which will be returned from `from_header`.
    type Error;

    /// The name of HTTP header associated with this type.
    const NAME: &'static str;

    /// The flag whether the endpoint will skip the request if the header value is missing.
    ///
    /// If the value of this flag is `false`, the endpoint will always accept the request
    /// and will return an error if the header value is missing.
    const ALLOW_SKIP: bool = true;

    /// Perform conversion from a byte sequence to a value of `Self`.
    fn from_header(s: &[u8]) -> Result<Self, Self::Error>;

    /// Return the default value of Self used if the header value is missing.
    ///
    /// If the returned value is `None`, it means that the header value is required and
    /// an error will be returned from the endpoint if the value is missing.
    fn default() -> Option<Self> {
        None
    }
}
