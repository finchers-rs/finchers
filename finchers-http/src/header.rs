//! Components for accessing of HTTP headers

use std::fmt;
use std::marker::PhantomData;

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};
use finchers_core::{HttpError, Never};

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint will always accept the request even if the header
/// value is not exist.
///
/// # Example
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_http::header::{header, FromHeader};
/// # use finchers_endpoint::{EndpointExt, EndpointResultExt, EndpointOptionExt};
/// # use finchers_core::error::BadRequest;
/// #
/// pub struct APIKey(pub String);
///
/// impl FromHeader for APIKey {
///     type Error = BadRequest;
///
///     fn header_name() -> &'static str { "X-API-Key" }
///
///     fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
///         String::from_utf8(s.to_owned())
///             .map(APIKey)
///             .map_err(|_| BadRequest::new("Invalid API key"))
///     }
/// }
///
/// # fn main() {
/// // impl Endpoint<Item = APIKey>
/// let api_key = header()
///     .ok_or_else(|| BadRequest::new("Missing API key"))
///     .unwrap_ok()
///     .as_t::<APIKey>();
/// # }
/// ```
pub fn header<H>() -> Header<H>
where
    H: FromHeader,
    H::Error: HttpError,
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
    H: FromHeader,
    H::Error: HttpError,
{
    type Output = Option<H>;
    type Outcome = HeaderOutcome<H>;

    fn apply(&self, _: &mut Context) -> Option<Self::Outcome> {
        Some(HeaderOutcome { _marker: PhantomData })
    }
}

#[doc(hidden)]
pub struct HeaderOutcome<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Outcome for HeaderOutcome<H>
where
    H: FromHeader,
    H::Error: HttpError,
{
    type Output = Option<H>;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        match cx.input().request().headers().get(H::header_name()) {
            Some(h) => match H::from_header(h.as_bytes()) {
                Ok(h) => PollOutcome::Ready(Some(h)),
                Err(e) => PollOutcome::Abort(Into::into(e)),
            },
            None => PollOutcome::Ready(None),
        }
    }
}

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint will perform convesion the header value *before* creating
/// a task. If the conversion is failed or the header value is not exist,
/// it will skip the request.
///
/// # Example
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_http::header::{header_skipped, FromHeader};
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_core::error::BadRequest;
/// #
/// pub struct APIKey(pub String);
///
/// impl FromHeader for APIKey {
///     type Error = BadRequest;
///
///     fn header_name() -> &'static str { "X-API-Key" }
///
///     fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
///         String::from_utf8(s.to_owned())
///             .map(APIKey)
///             .map_err(|_| BadRequest::new("invalid API key"))
///     }
/// }
///
/// # fn main() {
/// let api_key = header_skipped().map(|APIKey(key)| key);
/// # }
/// ```
pub fn header_skipped<H>() -> HeaderSkipped<H>
where
    H: FromHeader + Send,
{
    HeaderSkipped { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct HeaderSkipped<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for HeaderSkipped<H> {}

impl<H> Clone for HeaderSkipped<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for HeaderSkipped<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderSkipped").finish()
    }
}

impl<H> Endpoint for HeaderSkipped<H>
where
    H: FromHeader + Send,
{
    type Output = H;
    type Outcome = outcome::Ready<H>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        cx.input()
            .request()
            .headers()
            .get(H::header_name())
            .and_then(|h| H::from_header(h.as_bytes()).ok())
            .map(outcome::ready)
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
    type Error = Never;

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
    type Error = Never;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s))
    }
}
