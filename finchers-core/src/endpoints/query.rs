//! Components for parsing the query string and urlencoded payload.

use std::future::Future;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::ops::Deref;
use std::task::Poll;
use std::{fmt, task};

use bytes::Bytes;
use failure::{Fail, SyncFailure};
use http::StatusCode;
use serde::de::{self, DeserializeOwned, IntoDeserializer};
use {mime, serde_qs};

use endpoint::EndpointBase;
use error::HttpError;
use generic::{one, One};
use input::{with_get_cx, Cursor, FromBody, FromQuery, Input, QueryItems};

/// Create an endpoint which parse the query string in the HTTP request
/// to the value of `T`.
///
/// # Example
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # #![feature(use_extern_macros)]
/// # extern crate finchers_core;
/// # extern crate serde;
/// # use finchers_core::endpoints::path::path;
/// # use finchers_core::endpoints::query::{query, from_csv, Serde};
/// # use finchers_core::endpoint::EndpointExt;
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
/// let endpoint = path("foo").and(query())
///     .map_ok(|param: Serde<Param>| (format!("Received: {:?}", &*param),));
/// ```
pub fn query<T>() -> Query<T>
where
    T: FromQuery,
{
    Query {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Query<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Query<T> {}

impl<T> Clone for Query<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Query<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Query").finish()
    }
}

impl<T> EndpointBase for Query<T>
where
    T: FromQuery,
{
    type Ok = One<T>;
    type Error = QueryError<T::Error>;
    type Future = QueryFuture<T>;

    fn apply(&self, _: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        Some((
            QueryFuture {
                _marker: PhantomData,
            },
            cursor,
        ))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct QueryFuture<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Future for QueryFuture<T>
where
    T: FromQuery,
{
    type Output = Result<One<T>, QueryError<T::Error>>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| match input.request().uri().query() {
            Some(query) => T::from_query(QueryItems::new(query))
                .map(one)
                .map_err(|cause| QueryError::Parse { cause }),
            None => Err(QueryError::MissingQuery),
        }))
    }
}

/// A wrapper struct which contains a parsed content from the urlencoded string.
#[derive(Debug)]
pub struct Form<F>(pub F);

impl<F> Form<F> {
    /// Consume itself and return the instance of inner value.
    pub fn into_inner(self) -> F {
        self.0
    }
}

impl<F> Deref for Form<F> {
    type Target = F;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> FromBody for Form<F>
where
    F: FromQuery + 'static,
{
    type Error = QueryError<F::Error>;

    fn from_body(body: Bytes, input: PinMut<Input>) -> Result<Self, Self::Error> {
        if !input
            .content_type()
            .map_err(|_| QueryError::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            return Err(QueryError::InvalidMediaType);
        }

        FromQuery::from_query(QueryItems::new(&*body))
            .map(Form)
            .map_err(|cause| QueryError::Parse { cause })
    }
}

/// All of error kinds when receiving/parsing the urlencoded data.
#[derive(Debug)]
pub enum QueryError<E> {
    #[allow(missing_docs)]
    MissingQuery,
    #[allow(missing_docs)]
    InvalidMediaType,
    #[allow(missing_docs)]
    Parse { cause: E },
}

impl<E: fmt::Display> fmt::Display for QueryError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryError::MissingQuery => {
                write!(formatter, "The query string is not exist in the request")
            }
            QueryError::InvalidMediaType => write!(
                formatter,
                "The content type must be application/www-x-urlformencoded"
            ),
            QueryError::Parse { ref cause } => write!(formatter, "{}", cause),
        }
    }
}

impl<E: Fail> Fail for QueryError<E> {}

impl<E: HttpError> HttpError for QueryError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// A wrapper struct to add the implementation of `FromQuery` to `Deserialize`able types.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Serde<T>(pub T);

impl<T> Serde<T> {
    /// Consume itself and return the inner data of `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromQuery for Serde<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = SyncFailure<serde_qs::Error>;

    #[inline]
    fn from_query(query: QueryItems) -> Result<Self, Self::Error> {
        serde_qs::from_bytes(query.as_slice())
            .map(Serde)
            .map_err(SyncFailure::new)
    }
}

#[allow(missing_debug_implementations)]
struct CSVSeqVisitor<I, T> {
    _marker: PhantomData<fn() -> (I, T)>,
}

impl<'de, I, T> de::Visitor<'de> for CSVSeqVisitor<I, T>
where
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    type Value = I;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        s.split(",")
            .map(|s| de::Deserialize::deserialize(s.into_deserializer()))
            .collect()
    }
}

/// Deserialize a comma-separated string to a sequence of `T`.
///
/// This function is typically used as the attribute in the derivation of `serde::Deserialize`.
pub fn from_csv<'de, D, I, T>(de: D) -> Result<I, D::Error>
where
    D: de::Deserializer<'de>,
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    de.deserialize_str(CSVSeqVisitor {
        _marker: PhantomData,
    })
}
