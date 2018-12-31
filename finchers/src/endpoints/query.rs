//! Endpoints for parsing query strings.

use failure::SyncFailure;
use serde::de::DeserializeOwned;
use serde_qs;

use crate::endpoint::{ApplyError, Endpoint};
use crate::error;
use crate::error::bad_request;
use crate::future::EndpointFuture;

// ==== Required ====

/// Create an endpoint which parses the query string to the specified type.
///
/// If the query string is missing, this endpoint will skip the current request.
///
/// # Example
///
/// ```
/// # extern crate finchers;
/// # #[macro_use]
/// # extern crate serde;
/// # use finchers::endpoints::query;
/// # use finchers::prelude::*;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Param {
///     query: String,
///     count: Option<u32>,
/// }
///
/// # fn main() {
/// let endpoint = query::required()
///     .map(|param: Param| {
///         format!("Received: {:?}", param)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn required<T>() -> impl Endpoint<
    Output = (T,),
    Future = impl EndpointFuture<Output = (T,)> + Send + 'static, //
>
where
    T: DeserializeOwned + 'static,
{
    crate::endpoint::apply_fn(|cx| {
        if cx.uri().query().is_some() {
            Ok(crate::future::poll_fn(|input| {
                let query = input
                    .uri()
                    .query()
                    .expect("The query string should be available inside of this future.");
                serde_qs::from_str(query)
                    .map(|x| (x,).into())
                    .map_err(SyncFailure::new)
                    .map_err(bad_request)
            }))
        } else {
            Err(ApplyError::custom(error::bad_request("missing query")))
        }
    })
}

// ==== Optional ====

/// Create an endpoint which parses the query string to the specified type.
///
/// This endpoint always matches and returns a `None` if the query string is missing.
///
/// # Example
///
/// ```
/// # extern crate finchers;
/// # #[macro_use]
/// # extern crate serde;
/// # use finchers::endpoints::query;
/// # use finchers::prelude::*;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Param {
///     query: String,
///     count: Option<u32>,
/// }
///
/// # fn main() {
/// let endpoint = query::optional()
///     .map(|param: Option<Param>| {
///         format!("Received: {:?}", param)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn optional<T>() -> impl Endpoint<
    Output = (Option<T>,),
    Future = impl EndpointFuture<Output = (Option<T>,)> + Send + 'static, //
>
where
    T: DeserializeOwned + 'static,
{
    crate::endpoint::apply_fn(|_| {
        Ok(crate::future::poll_fn(|cx| match cx.uri().query() {
            Some(query) => serde_qs::from_str(query)
                .map(|x| (Some(x),).into())
                .map_err(|err| bad_request(SyncFailure::new(err))),
            None => Ok((None,).into()),
        }))
    })
}

/// Create an endpoint which extracts the query string from a request.
pub fn raw() -> impl Endpoint<
    Output = (Option<String>,),
    Future = impl EndpointFuture<Output = (Option<String>,)> + Send + 'static, //
> {
    crate::endpoint::apply_fn(|_| {
        Ok(crate::future::poll_fn(|cx| {
            let raw = cx.uri().query().map(ToOwned::to_owned);
            Ok::<_, crate::error::Never>((raw,).into())
        }))
    })
}
