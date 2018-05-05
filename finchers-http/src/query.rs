//! Components for parsing the query string and urlencoded payload.

use bytes::Bytes;
use failure::SyncFailure;
use http::StatusCode;
use serde::de::{self, IntoDeserializer};
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use {mime, serde_qs};

use body::FromBody;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, HttpError, Input, Poll, PollResult};

/// Create an endpoint which parse the query string in the HTTP request
/// to the value of "T".
///
/// # Example
///
/// ```
/// # extern crate finchers_ext;
/// # extern crate finchers_http;
/// # #[macro_use] extern crate serde;
/// # use finchers_http::query::{query, from_csv};
/// # use finchers_ext::EndpointExt;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Param {
///     query: String,
///     count: Option<u32>,
///     #[serde(deserialize_with = "from_csv", default)]
///     tags: Vec<String>,
/// }
///
/// # fn main() {
/// let endpoint = query()
///     .map(|param: Param| format!("Received: {:?}", param));
/// # }
/// ```
pub fn query<T>() -> Query<T>
where
    T: de::DeserializeOwned,
{
    Query { _marker: PhantomData }
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

impl<T> Endpoint for Query<T>
where
    T: de::DeserializeOwned,
{
    type Output = T;
    type Task = QueryTask<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        if cx.input().request().uri().query().is_some() {
            Some(QueryTask { _marker: PhantomData })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct QueryTask<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Task for QueryTask<T>
where
    T: de::DeserializeOwned,
{
    type Output = T;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        let query = cx.input()
            .request()
            .uri()
            .query()
            .expect("The query string should be exist at this location");
        match serde_qs::from_str(query) {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(e) => Poll::Ready(Err(QueryError::Parsing {
                cause: SyncFailure::new(e),
            }.into())),
        }
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
    F: de::DeserializeOwned + 'static,
{
    type Error = QueryError;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_err(|_| QueryError::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            serde_qs::from_bytes(&*body).map(Form).map_err(|e| QueryError::Parsing {
                cause: SyncFailure::new(e),
            })
        } else {
            Err(QueryError::InvalidMediaType)
        }
    }
}

/// All of error kinds when receiving/parsing the urlencoded data.
#[derive(Debug, Fail)]
pub enum QueryError {
    #[fail(display = "The content type should be application/www-x-urlformencoded")]
    InvalidMediaType,

    #[fail(display = "{}", cause)]
    Parsing { cause: SyncFailure<serde_qs::Error> },
}

impl HttpError for QueryError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
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

/// Deserialize a sequece from a comma-separated string.
///
/// This function is typically used in an attribute of the derivation "Deserialize".
///
/// # Example
///
/// ```
/// # extern crate finchers_http;
/// # extern crate finchers_ext;
/// # #[macro_use] extern crate serde;
/// # use finchers_http::query::{query, from_csv};
/// # use finchers_ext::EndpointExt;
/// #
/// #[derive(Debug, Deserialize)]
/// pub struct Params {
///     #[serde(deserialize_with = "from_csv", default)]
///     tags: Vec<String>,
/// }
///
/// # fn main() {
/// let endpoint = query::<Params>()
///     .inspect(|params| println!("{:?}", params.tags));
/// # }
/// ```
pub fn from_csv<'de, D, I, T>(de: D) -> Result<I, D::Error>
where
    D: de::Deserializer<'de>,
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    de.deserialize_str(CSVSeqVisitor { _marker: PhantomData })
}
