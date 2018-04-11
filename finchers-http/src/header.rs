//! Components for accessing of HTTP headers

use finchers_core::endpoint::{Context, Endpoint, Error};
use finchers_core::error::NotPresent;
use finchers_core::{HttpError, Input};
use futures::future::{err, ok, FutureResult, IntoFuture};
use std::fmt;
use std::marker::PhantomData;

/// Create an endpoint which parses an entry in the HTTP header.
///
/// If the entry is not given or the conversion is failed, this endpoint
/// will skip the request.
///
/// # Example
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_http::header::{header, FromHeader};
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_core::error::BadRequest;
/// # use std::string::FromUtf8Error;
/// #
/// pub struct APIKey(pub String);
///
/// impl FromHeader for APIKey {
///     type Error = BadRequest<FromUtf8Error>;
///
///     fn header_name() -> &'static str { "X-API-Key" }
///
///     fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
///         String::from_utf8(s.to_owned())
///             .map(APIKey)
///             .map_err(BadRequest::new)
///     }
/// }
///
/// # fn main() {
/// let api_key = header().map(|APIKey(key)| key);
/// # }
/// ```
///
/// By default, the error occuring when performing conversion to "H" is
/// interpreted as "should be skipped". You could change this behaviour
/// by composing some combinators as follows:
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_http::header::{header, FromHeader};
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_core::error::BadRequest;
/// # pub struct APIKey(pub String);
/// # impl FromHeader for APIKey {
/// #    type Error = !;
/// #    fn header_name() -> &'static str { "X-API-Key" }
/// #    fn from_header(s: &[u8]) -> Result<Self, Self::Error> { unimplemented!() }
/// # }
/// # fn main() {
/// let api_key = header::<Result<APIKey, _>>()
///     .try_abort(|key| key);
/// # }
/// ```
pub fn header<H>() -> Header<H>
where
    H: FromHeader + Send,
{
    Header { _marker: PhantomData }
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

impl<H> Endpoint for Header<H>
where
    H: FromHeader + Send,
{
    type Item = H;
    type Future = FutureResult<H, Error>;

    fn apply(&self, input: &Input, _: &mut Context) -> Option<Self::Future> {
        input
            .headers()
            .get(H::header_name())
            .and_then(|h| H::from_header(h.as_bytes()).ok())
            .map(ok)
    }
}

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint will abort handling the request if the header does
/// not exist or the conversion to "H" is failed.
///
/// # Example
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_http::header::{header_required, FromHeader};
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_core::error::BadRequest;
/// # use std::string::FromUtf8Error;
/// #
/// pub struct APIKey(pub String);
///
/// impl FromHeader for APIKey {
///     type Error = BadRequest<FromUtf8Error>;
///
///     fn header_name() -> &'static str { "X-API-Key" }
///
///     fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
///         String::from_utf8(s.to_owned())
///             .map(APIKey)
///             .map_err(BadRequest::new)
///     }
/// }
///
/// # fn main() {
/// let api_key = header_required().map(|APIKey(key)| key);
/// # }
/// ```
pub fn header_required<H>() -> HeaderRequired<H>
where
    H: FromHeader + Send,
    H::Error: HttpError,
{
    HeaderRequired { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct HeaderRequired<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for HeaderRequired<H> {}

impl<H> Clone for HeaderRequired<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for HeaderRequired<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderRequired").finish()
    }
}

impl<H> Endpoint for HeaderRequired<H>
where
    H: FromHeader + Send,
    H::Error: HttpError,
{
    type Item = H;
    type Future = FutureResult<H, Error>;

    fn apply(&self, input: &Input, _: &mut Context) -> Option<Self::Future> {
        match input.headers().get(H::header_name()) {
            Some(h) => Some(H::from_header(h.as_bytes()).map_err(Into::into).into_future()),
            None => Some(err(NotPresent::new("").into())),
        }
    }
}

/// Trait representing the conversion from an entry of HTTP header.
pub trait FromHeader: 'static + Sized {
    /// The error type which will be returned from "from_header".
    type Error;

    /// Return the name of HTTP header associated with this type.
    fn header_name() -> &'static str;

    /// Perform conversion from a bytes to "Self".
    fn from_header(s: &[u8]) -> Result<Self, Self::Error>;
}

impl<H: FromHeader> FromHeader for Option<H> {
    type Error = !;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s).ok())
    }
}

impl<H: FromHeader> FromHeader for Result<H, H::Error> {
    type Error = !;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s))
    }
}
