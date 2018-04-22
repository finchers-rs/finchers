//! Components for accessing of HTTP headers

use futures::future::{ok, FutureResult};
use std::fmt;
use std::marker::PhantomData;

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, CompatTask, PollTask, Task};
use finchers_core::{Error, HttpError};

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
///             .map_err(|e| BadRequest::new("Invalid API key").with_cause(e))
///     }
/// }
///
/// # fn main() {
/// // impl Endpoint<Item = APIKey>
/// let api_key = header::<APIKey>().try_abort(|h| {
///     h.ok_or_else(|| BadRequest::new("Missing API key"))
/// });
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
    type Item = Option<H>;
    type Task = HeaderTask<H>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(HeaderTask { _marker: PhantomData })
    }
}

#[doc(hidden)]
pub struct HeaderTask<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Task for HeaderTask<H>
where
    H: FromHeader,
    H::Error: HttpError,
{
    type Output = Option<H>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let header = cx.input().request().headers().get(H::header_name());
        match header {
            Some(h) => H::from_header(h.as_bytes()).map(|h| Some(h).into()).map_err(Into::into),
            None => Ok(None.into()),
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
///             .map_err(|e| BadRequest::new("invalid API key").with_cause(e))
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
    type Item = H;
    type Task = CompatTask<FutureResult<H, Error>>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        cx.input()
            .request()
            .headers()
            .get(H::header_name())
            .and_then(|h| H::from_header(h.as_bytes()).ok())
            .map(ok)
            .map(CompatTask::from)
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
