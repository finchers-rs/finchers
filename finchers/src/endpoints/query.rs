//! Endpoints for parsing query strings.

use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            ApplyError,
            ApplyResult,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::{BadRequest, Error},
    },
    failure::SyncFailure,
    futures::Poll,
    serde::de::DeserializeOwned,
    std::marker::PhantomData,
};

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
        type Action = RequiredAction<T>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            if cx.uri().query().is_some() {
                Ok(RequiredAction {
                    _marker: PhantomData,
                })
            } else {
                Err(ApplyError::custom(BadRequest::from("missing query")))
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RequiredAction<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointAction<Bd> for RequiredAction<T>
    where
        T: DeserializeOwned,
    {
        type Output = (T,);

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
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
        type Action = OptionalAction<T>;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            Ok(OptionalAction {
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct OptionalAction<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointAction<Bd> for OptionalAction<T>
    where
        T: DeserializeOwned,
    {
        type Output = (Option<T>,);

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
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
        type Action = RawAction;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            Ok(RawAction(()))
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RawAction(());

    impl<Bd> EndpointAction<Bd> for RawAction {
        type Output = (Option<String>,);

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            let raw = cx.uri().query().map(ToOwned::to_owned);
            Ok((raw,).into())
        }
    }
}
