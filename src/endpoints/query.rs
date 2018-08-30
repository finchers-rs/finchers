//! Endpoints for parsing query strings.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::{bad_request, Error};
use crate::input::query::{FromQuery, QueryItems};
use crate::input::with_get_cx;

// ==== Required ====

/// Create an endpoint which parses the query string to the specified type.
///
/// If the query string is missing, this endpoint will return an error.
///
/// # Example
///
/// ```
/// # extern crate finchers;
/// # extern crate serde;
/// # use finchers::endpoints::query;
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::input::query::{from_csv, Serde};
/// # use serde::Deserialize;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Param {
///     query: String,
///     count: Option<u32>,
///     #[serde(deserialize_with = "from_csv", default)]
///     tags: Vec<String>,
/// }
///
/// let endpoint = query::required()
///     .map(|param: Serde<Param>| {
///         format!("Received: {:?}", param)
///     });
/// # drop(endpoint);
/// ```
pub fn required<T>() -> Required<T>
where
    T: FromQuery,
{
    Required {
        _marker: PhantomData,
    }
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
        f.debug_struct("Parse").finish()
    }
}

impl<'a, T> Endpoint<'a> for Required<T>
where
    T: FromQuery,
{
    type Output = (T,);
    type Future = RequiredFuture<T>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(RequiredFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RequiredFuture<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Future for RequiredFuture<T>
where
    T: FromQuery,
{
    type Output = Result<(T,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| match input.request().uri().query() {
            Some(query) => {
                let items = unsafe { QueryItems::new_unchecked(query) };
                T::from_query(items).map(|x| (x,)).map_err(bad_request)
            }
            None => Err(bad_request("missing query string")),
        }))
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
/// # extern crate serde;
/// # use finchers::endpoints::query;
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::input::query::{from_csv, Serde};
/// # use serde::Deserialize;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Param {
///     query: String,
///     count: Option<u32>,
///     #[serde(deserialize_with = "from_csv", default)]
///     tags: Vec<String>,
/// }
///
/// let endpoint = query::optional()
///     .map(|param: Option<Serde<Param>>| {
///         format!("Received: {:?}", param)
///     });
/// # drop(endpoint);
/// ```
pub fn optional<T>() -> Optional<T>
where
    T: FromQuery,
{
    Optional {
        _marker: PhantomData,
    }
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
    T: FromQuery,
{
    type Output = (Option<T>,);
    type Future = OptionalFuture<T>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
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

impl<T> Future for OptionalFuture<T>
where
    T: FromQuery,
{
    type Output = Result<(Option<T>,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| match input.request().uri().query() {
            Some(query) => {
                let items = unsafe { QueryItems::new_unchecked(query) };
                T::from_query(items)
                    .map(|x| (Some(x),))
                    .map_err(bad_request)
            }
            None => Ok((None,)),
        }))
    }
}

/// Create an endpoint which extracts the query string from a request.
pub fn raw() -> Raw {
    Raw { _priv: () }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Raw {
    _priv: (),
}

impl<'a> Endpoint<'a> for Raw {
    type Output = (Option<String>,);
    type Future = RawFuture;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(RawFuture { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RawFuture {
    _priv: (),
}

impl Future for RawFuture {
    type Output = Result<(Option<String>,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        let raw = with_get_cx(|input| input.request().uri().query().map(ToOwned::to_owned));
        Poll::Ready(Ok((raw,)))
    }
}
