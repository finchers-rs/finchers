//! Endpoints for parsing query strings.

use failure::SyncFailure;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use crate::endpoint::{ApplyContext, ApplyError, ApplyResult, Endpoint, IsEndpoint};
use crate::error::{BadRequest, Error};
use crate::future::{Context, EndpointFuture, Poll};

// ==== Required ====

/// Create an endpoint which parses the query string to the specified type.
///
/// If the query string is missing, this endpoint will skip the current request.
///
/// # Example
///
/// ```ignore
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
    T: DeserializeOwned,
{
    Required {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Required<T> {
    _marker: PhantomData<fn() -> T>,
}

mod required {
    use super::*;

    impl<T: DeserializeOwned> IsEndpoint for Required<T> {}

    impl<T, Bd> Endpoint<Bd> for Required<T>
    where
        T: DeserializeOwned,
    {
        type Output = (T,);
        type Future = RequiredFuture<T>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            if cx.uri().query().is_some() {
                Ok(RequiredFuture {
                    _marker: PhantomData,
                })
            } else {
                Err(ApplyError::custom(BadRequest::from("missing query")))
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RequiredFuture<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointFuture<Bd> for RequiredFuture<T>
    where
        T: DeserializeOwned,
    {
        type Output = (T,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            let query = cx
                .uri()
                .query()
                .expect("The query string should be available inside of this future.");
            serde_qs::from_str(query)
                .map(|x| (x,).into())
                .map_err(SyncFailure::new)
                .map_err(BadRequest::from)
                .map_err(Into::into)
        }
    }
}

// ==== Optional ====

/// Create an endpoint which parses the query string to the specified type.
///
/// This endpoint always matches and returns a `None` if the query string is missing.
///
/// # Example
///
/// ```ignore
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
    T: DeserializeOwned,
{
    Optional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Optional<T> {
    _marker: PhantomData<fn() -> T>,
}

mod optional {
    use super::*;

    impl<T: DeserializeOwned> IsEndpoint for Optional<T> {}

    impl<T, Bd> Endpoint<Bd> for Optional<T>
    where
        T: DeserializeOwned,
    {
        type Output = (Option<T>,);
        type Future = OptionalFuture<T>;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            Ok(OptionalFuture {
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct OptionalFuture<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointFuture<Bd> for OptionalFuture<T>
    where
        T: DeserializeOwned,
    {
        type Output = (Option<T>,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            match cx.uri().query() {
                Some(query) => serde_qs::from_str(query)
                    .map(|x| (Some(x),).into())
                    .map_err(|err| BadRequest::from(SyncFailure::new(err)))
                    .map_err(Into::into),
                None => Ok((None,).into()),
            }
        }
    }

}

/// Create an endpoint which extracts the query string from a request.
pub fn raw() -> Raw {
    Raw(())
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Raw(());

mod raw {
    use super::*;

    impl IsEndpoint for Raw {}

    impl<Bd> Endpoint<Bd> for Raw {
        type Output = (Option<String>,);
        type Future = RawFuture;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            Ok(RawFuture(()))
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RawFuture(());

    impl<Bd> EndpointFuture<Bd> for RawFuture {
        type Output = (Option<String>,);

        fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            let raw = cx.uri().query().map(ToOwned::to_owned);
            Ok((raw,).into())
        }
    }
}
