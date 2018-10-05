//! Endpoints for parsing query strings.

use failure::SyncFailure;
use serde::de::DeserializeOwned;
use serde_qs;
use std::fmt;
use std::marker::PhantomData;

use endpoint::with_get_cx;
use endpoint::{ApplyContext, ApplyError, ApplyResult, Endpoint};
use error;
use error::{bad_request, Error};

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
pub fn required<T>() -> Required<T>
where
    T: DeserializeOwned + 'static,
{
    (Required {
        _marker: PhantomData,
    }).with_output::<(T,)>()
}

#[allow(missing_docs)]
pub struct Required<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Required<T> {}

impl<T> Clone for Required<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Required<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Required").finish()
    }
}

impl<'a, T> Endpoint<'a> for Required<T>
where
    T: DeserializeOwned + 'static,
{
    type Output = (T,);
    type Future = RequiredFuture<T>;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        if cx.input().uri().query().is_some() {
            Ok(RequiredFuture {
                _marker: PhantomData,
            })
        } else {
            Err(ApplyError::custom(error::bad_request("missing query")))
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RequiredFuture<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> ::futures::Future for RequiredFuture<T>
where
    T: DeserializeOwned + 'static,
{
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        with_get_cx(|input| {
            let query = input
                .uri()
                .query()
                .expect("The query string should be available inside of this future.");
            serde_qs::from_str(query).map_err(SyncFailure::new)
        }).map(|x| (x,).into())
        .map_err(bad_request)
    }
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
pub fn optional<T>() -> Optional<T>
where
    T: DeserializeOwned + 'static,
{
    (Optional {
        _marker: PhantomData,
    }).with_output::<(Option<T>,)>()
}

#[allow(missing_docs)]
pub struct Optional<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Optional<T> {}

impl<T> Clone for Optional<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Optional<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Optional").finish()
    }
}

impl<'a, T> Endpoint<'a> for Optional<T>
where
    T: DeserializeOwned + 'static,
{
    type Output = (Option<T>,);
    type Future = OptionalFuture<T>;

    fn apply(&self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(OptionalFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct OptionalFuture<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> ::futures::Future for OptionalFuture<T>
where
    T: DeserializeOwned + 'static,
{
    type Item = (Option<T>,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        with_get_cx(|input| match input.uri().query() {
            Some(query) => serde_qs::from_str(query)
                .map(|x| (Some(x),).into())
                .map_err(|err| bad_request(SyncFailure::new(err))),
            None => Ok((None,).into()),
        })
    }
}

/// Create an endpoint which extracts the query string from a request.
pub fn raw() -> Raw {
    (Raw { _priv: () }).with_output::<(Option<String>,)>()
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Raw {
    _priv: (),
}

impl<'a> Endpoint<'a> for Raw {
    type Output = (Option<String>,);
    type Future = RawFuture;

    fn apply(&self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(RawFuture { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawFuture {
    _priv: (),
}

impl ::futures::Future for RawFuture {
    type Item = (Option<String>,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        let raw = with_get_cx(|input| input.uri().query().map(ToOwned::to_owned));
        Ok((raw,).into())
    }
}
