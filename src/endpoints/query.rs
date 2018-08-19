//! Components for parsing the query string and urlencoded payload.

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::task::Poll;
use std::{fmt, task};

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::{bad_request, Error};
use crate::generic::{one, One};
use crate::input::query::{FromQuery, QueryItems};
use crate::input::with_get_cx;

/// Create an endpoint which parse the query string in the HTTP request
/// to the value of `T`.
///
/// # Example
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # #![feature(use_extern_macros)]
/// # extern crate finchers;
/// # extern crate serde;
/// # use finchers::endpoints::path::path;
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
/// let endpoint = path("foo").and(query::parse())
///     .map(|param: Serde<Param>| (format!("Received: {:?}", &*param),));
/// ```
pub fn parse<T>() -> Parse<T>
where
    T: FromQuery,
{
    Parse {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Parse<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Parse<T> {}

impl<T> Clone for Parse<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse").finish()
    }
}

impl<T> Endpoint for Parse<T>
where
    T: FromQuery,
{
    type Output = One<T>;
    type Future = ParseFuture<T>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(ParseFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ParseFuture<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Future for ParseFuture<T>
where
    T: FromQuery,
{
    type Output = Result<One<T>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            let items = match input.request().uri().query() {
                Some(query) => unsafe { QueryItems::new_unchecked(query) },
                None => QueryItems::empty(),
            };
            T::from_query(items).map(one).map_err(bad_request)
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

impl Endpoint for Raw {
    type Output = One<String>;
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
    type Output = Result<One<String>, Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(one(with_get_cx(|input| {
            input
                .request()
                .uri()
                .query()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| "".into())
        }))))
    }
}
