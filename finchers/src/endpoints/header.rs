//! Components for parsing the HTTP headers.

use http::header::{HeaderName, HeaderValue};
use http::HttpTryFrom;
use std::fmt;

use crate::endpoint::{ApplyError, Endpoint};
use crate::error;
use crate::future::EndpointFuture;
use crate::input::FromHeaderValue;

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
pub fn parse<T>(
    name: &'static str,
) -> impl Endpoint<
    Output = (T,),
    Future = impl EndpointFuture<Output = (T,)> + Send + 'static, // private
>
where
    T: FromHeaderValue,
{
    let name = HeaderName::from_static(name);

    crate::endpoint::apply_fn(move |cx| {
        if cx.input().headers().contains_key(&name) {
            let name = name.clone();
            Ok(crate::future::poll_fn(move |cx| {
                let h = cx
                    .headers()
                    .get(&name)
                    .expect("The header value should be always available inside of this Future.");
                T::from_header_value(h)
                    .map(|parsed| (parsed,).into())
                    .map_err(error::bad_request)
            }))
        } else {
            Err(ApplyError::custom(error::bad_request(format!(
                "missing header: `{}'",
                name.as_str()
            ))))
        }
    })
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
pub fn optional<T>(
    name: &'static str,
) -> impl Endpoint<
    Output = (Option<T>,),
    Future = impl EndpointFuture<Output = (Option<T>,)> + Send + 'static, // private
>
where
    T: FromHeaderValue,
{
    let name = HeaderName::from_static(name);
    crate::endpoint::apply_fn(move |_| {
        let name = name.clone();
        Ok(crate::future::poll_fn(move |cx| {
            match cx.headers().get(&name) {
                Some(h) => T::from_header_value(h)
                    .map(|parsed| (Some(parsed),).into())
                    .map_err(error::bad_request),
                None => Ok((None,).into()),
            }
        }))
    })
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
pub fn matches<K, V>(
    name: K,
    value: V,
) -> impl Endpoint<
    Output = (),
    Future = impl EndpointFuture<Output = ()> + Send + 'static, // private
>
where
    HeaderName: HttpTryFrom<K>,
    <HeaderName as HttpTryFrom<K>>::Error: fmt::Debug,
    V: PartialEq<HeaderValue>,
{
    let name = HeaderName::try_from(name).expect("invalid header name");
    crate::endpoint::apply_fn(move |cx| match cx.headers().get(&name) {
        Some(v) if value == *v => Ok(crate::future::poll_fn(|_| {
            Ok::<_, crate::error::Never>(().into())
        })),
        _ => Err(ApplyError::not_matched()),
    })
}

// ==== Raw ====

/// Create an endpoint which retrieves the value of a header with the specified name.
#[inline]
pub fn raw<H>(
    name: H,
) -> impl Endpoint<
    Output = (Option<HeaderValue>,),
    Future = impl EndpointFuture<Output = (Option<HeaderValue>,)> + Send + 'static, //
>
where
    HeaderName: HttpTryFrom<H>,
    <HeaderName as HttpTryFrom<H>>::Error: fmt::Debug,
{
    let name = HeaderName::try_from(name).expect("invalid header name");
    crate::endpoint::apply_fn(move |cx| {
        let mut value = cx.headers().get(&name).cloned();
        Ok(crate::future::poll_fn(move |_| {
            Ok::<_, crate::error::Never>((value.take(),).into())
        }))
    })
}
