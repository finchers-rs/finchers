//! Components for parsing the HTTP headers.

use std::fmt;
use std::marker::PhantomData;
use std::pin::PinMut;

use futures_core::future::Future;
use futures_core::task;
use futures_core::task::Poll;
use futures_util::future;

use http::header::{HeaderName, HeaderValue};
use http::HttpTryFrom;

use crate::endpoint::{Context, Endpoint, EndpointError, EndpointExt, EndpointResult};
use crate::error;
use crate::error::Error;
use crate::input::header::FromHeaderValue;
use crate::input::{with_get_cx, Input};

// ==== Parse ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::parse::<String>("x-api-key");
/// # drop(endpoint);
/// ```
///
/// By default, this endpoint will skip the current request if the specified
/// name of header does not exist. In order to trait the missing header as an
/// error, call `required()` as follows:
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::parse::<String>("x-api-key").required();
/// # drop(endpoint);
/// ```
pub fn parse<T>(name: &'static str) -> Parse<T>
where
    T: FromHeaderValue,
{
    (Parse {
        name: HeaderName::from_static(name),
        skip_if_missing: true,
        _marker: PhantomData,
    }).output::<(T,)>()
}

#[allow(missing_docs)]
pub struct Parse<T> {
    name: HeaderName,
    skip_if_missing: bool,
    _marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse")
            .field("name", &self.name)
            .field("skip_if_missing", &self.skip_if_missing)
            .finish()
    }
}

impl<T> Parse<T>
where
    T: FromHeaderValue,
{
    /// Change the settings of this endpoint to return an error
    /// without skipping when the value of specified header name
    /// does not exist in the request.
    pub fn required(self) -> Parse<T> {
        Parse {
            skip_if_missing: false,
            ..self
        }
    }

    fn parse_value(&self, input: &Input) -> Result<T, Error> {
        let h = input.request().headers().get(&self.name).ok_or_else(|| {
            let msg = format!("missing header: `{}'", self.name.as_str());
            error::bad_request(msg)
        })?;
        T::from_header_value(h).map_err(error::bad_request)
    }
}

impl<'e, T> Endpoint<'e> for Parse<T>
where
    T: FromHeaderValue,
{
    type Output = (T,);
    type Future = ParseFuture<'e, T>;

    fn apply(&'e self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        if self.skip_if_missing && !cx.input().headers().contains_key(&self.name) {
            return Err(EndpointError::not_matched());
        }
        Ok(ParseFuture { endpoint: self })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ParseFuture<'e, T: 'e> {
    endpoint: &'e Parse<T>,
}

impl<'e, T> Future for ParseFuture<'e, T>
where
    T: FromHeaderValue,
{
    type Output = Result<(T,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            self.endpoint.parse_value(&*input).map(|parsed| (parsed,))
        }))
    }
}

// ==== Optional ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint always matches to the request and it will return a `None`
/// if the value of specified header name is missing.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::optional::<String>("x-api-key");
/// # drop(endpoint);
/// ```
pub fn optional<T>(name: &'static str) -> Optional<T>
where
    T: FromHeaderValue,
{
    (Optional {
        name: HeaderName::from_static(name),
        _marker: PhantomData,
    }).output::<(Option<T>,)>()
}

#[allow(missing_docs)]
pub struct Optional<T> {
    name: HeaderName,
    _marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for Optional<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Optional")
            .field("name", &self.name)
            .finish()
    }
}

impl<'e, T> Endpoint<'e> for Optional<T>
where
    T: FromHeaderValue,
{
    type Output = (Option<T>,);
    type Future = OptionalFuture<'e, T>;

    fn apply(&'e self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(OptionalFuture { endpoint: self })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct OptionalFuture<'e, T: 'e> {
    endpoint: &'e Optional<T>,
}

impl<'e, T> Future for OptionalFuture<'e, T>
where
    T: FromHeaderValue,
{
    type Output = Result<(Option<T>,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            match input.request().headers().get(&self.endpoint.name) {
                Some(h) => T::from_header_value(h)
                    .map(|parsed| (Some(parsed),))
                    .map_err(error::bad_request),
                None => Ok((None,)),
            }
        }))
    }
}

// ==== Matches ====

/// Create an endpoint which checks if a header entry with the specified name
/// is equal to the specified value.
///
/// If the header with the given name does not exists or it is not equal to
/// the specified value, it treats the request as not matching.
///
/// # Examples
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::matches("origin", "www.example.com");
/// # drop(endpoint);
/// ```
///
/// ```
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::endpoints::header;
/// use finchers::endpoint::reject;
/// use finchers::error;
///
/// let endpoint = header::matches("origin", "www.example.com")
///     .or(reject(|_| error::bad_request("The value of Origin is invalid")));
/// # drop(endpoint);
/// ```
pub fn matches<K, V>(name: K, value: V) -> Matches<V>
where
    HeaderName: HttpTryFrom<K>,
    <HeaderName as HttpTryFrom<K>>::Error: fmt::Debug,
    V: PartialEq<HeaderValue>,
{
    (Matches {
        name: HeaderName::try_from(name).expect("invalid header name"),
        value,
    }).output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Matches<V> {
    name: HeaderName,
    value: V,
}

impl<'e, V> Endpoint<'e> for Matches<V>
where
    V: PartialEq<HeaderValue> + 'e,
{
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match ecx.input().headers().get(&self.name) {
            Some(value) if self.value == *value => Ok(future::ready(Ok(()))),
            _ => Err(EndpointError::not_matched()),
        }
    }
}

// ==== Raw ====

/// Create an endpoint which retrieves the value of a header with the specified name.
pub fn raw<H>(name: H) -> Raw
where
    HeaderName: HttpTryFrom<H>,
    <HeaderName as HttpTryFrom<H>>::Error: fmt::Debug,
{
    Raw {
        name: HeaderName::try_from(name).expect("invalid header name"),
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Raw {
    name: HeaderName,
}

impl<'a> Endpoint<'a> for Raw {
    type Output = (Option<HeaderValue>,);
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let header = cx.input().headers().get(&self.name).cloned();
        Ok(future::ready(Ok((header,))))
    }
}
