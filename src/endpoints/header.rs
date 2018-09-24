//! Components for parsing the HTTP headers.

use http::header::{HeaderName, HeaderValue};
use http::HttpTryFrom;
use std::fmt;
use std::marker::PhantomData;

use endpoint::{Context, Endpoint, EndpointError, EndpointResult};
use error;
use error::Error;
use input::with_get_cx;
use input::FromHeaderValue;

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
/// error, use `or_reject_with` as follows:
///
/// ```
/// # use finchers::prelude::*;
/// # use finchers::endpoints::header;
/// use finchers::endpoint::wrapper::or_reject_with;
/// use finchers::error::bad_request;
///
/// let endpoint = header::parse::<String>("x-api-key")
///     .wrap(or_reject_with(|_, _| bad_request("missing header: x-api-key")));
/// # drop(endpoint);
/// ```
#[inline]
pub fn parse<T>(name: &'static str) -> Parse<T>
where
    T: FromHeaderValue,
{
    (Parse {
        name: HeaderName::from_static(name),
        _marker: PhantomData,
    }).with_output::<(T,)>()
}

#[allow(missing_docs)]
pub struct Parse<T> {
    name: HeaderName,
    _marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse").field("name", &self.name).finish()
    }
}

impl<'e, T> Endpoint<'e> for Parse<T>
where
    T: FromHeaderValue,
{
    type Output = (T,);
    type Future = ParseFuture<'e, T>;

    fn apply(&'e self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        if cx.input().headers().contains_key(&self.name) {
            Ok(ParseFuture { endpoint: self })
        } else {
            Err(EndpointError::custom(error::bad_request(format!(
                "missing header: `{}'",
                self.name.as_str()
            ))))
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ParseFuture<'e, T: 'e> {
    endpoint: &'e Parse<T>,
}

impl<'e, T> ::futures::Future for ParseFuture<'e, T>
where
    T: FromHeaderValue,
{
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        with_get_cx(|input| {
            let h = input
                .request()
                .headers()
                .get(&self.endpoint.name)
                .expect("The header value should be always available inside of this Future.");
            T::from_header_value(h)
                .map(|parsed| (parsed,).into())
                .map_err(error::bad_request)
        })
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
#[inline]
pub fn optional<T>(name: &'static str) -> Optional<T>
where
    T: FromHeaderValue,
{
    (Optional {
        name: HeaderName::from_static(name),
        _marker: PhantomData,
    }).with_output::<(Option<T>,)>()
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

impl<'e, T> ::futures::Future for OptionalFuture<'e, T>
where
    T: FromHeaderValue,
{
    type Item = (Option<T>,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        with_get_cx(
            |input| match input.request().headers().get(&self.endpoint.name) {
                Some(h) => T::from_header_value(h)
                    .map(|parsed| (Some(parsed),).into())
                    .map_err(error::bad_request),
                None => Ok((None,).into()),
            },
        )
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
/// # use finchers::prelude::*;
/// # use finchers::endpoints::header;
/// use finchers::error;
///
/// let endpoint = header::matches("origin", "www.example.com")
///     .wrap(endpoint::wrapper::or_reject_with(|_, _| error::bad_request("invalid header value")));
/// # drop(endpoint);
/// ```
#[inline]
pub fn matches<K, V>(name: K, value: V) -> Matches<V>
where
    HeaderName: HttpTryFrom<K>,
    <HeaderName as HttpTryFrom<K>>::Error: fmt::Debug,
    V: PartialEq<HeaderValue>,
{
    (Matches {
        name: HeaderName::try_from(name).expect("invalid header name"),
        value,
    }).with_output::<()>()
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
    type Future = ::futures::future::FutureResult<Self::Output, Error>;

    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match ecx.input().headers().get(&self.name) {
            Some(value) if self.value == *value => Ok(::futures::future::result(Ok(()))),
            _ => Err(EndpointError::not_matched()),
        }
    }
}

// ==== Raw ====

/// Create an endpoint which retrieves the value of a header with the specified name.
#[inline]
pub fn raw<H>(name: H) -> Raw
where
    HeaderName: HttpTryFrom<H>,
    <HeaderName as HttpTryFrom<H>>::Error: fmt::Debug,
{
    (Raw {
        name: HeaderName::try_from(name).expect("invalid header name"),
    }).with_output::<(Option<HeaderValue>,)>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Raw {
    name: HeaderName,
}

impl<'a> Endpoint<'a> for Raw {
    type Output = (Option<HeaderValue>,);
    type Future = ::futures::future::FutureResult<Self::Output, Error>;

    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let header = cx.input().headers().get(&self.name).cloned();
        Ok(::futures::future::result(Ok((header,))))
    }
}
