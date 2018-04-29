//! Components for parsing the query string and urlencoded payload.

use bytes::Bytes;
use http::StatusCode;
use serde::de::{self, IntoDeserializer};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{error, fmt};
use {mime, serde_qs};

use body::FromData;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};
use finchers_core::{HttpError, Input};

/// Create an endpoint which parse the query string in the HTTP request
/// to the value of "T".
///
/// # Example
///
/// ```
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # #[macro_use] extern crate serde;
/// # use finchers_http::query::{query, from_csv};
/// # use finchers_endpoint::EndpointExt;
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
    type Outcome = QueryOutcome<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        if cx.input().request().uri().query().is_some() {
            Some(QueryOutcome { _marker: PhantomData })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct QueryOutcome<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Outcome for QueryOutcome<T>
where
    T: de::DeserializeOwned,
{
    type Output = T;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let query = cx.input()
            .request()
            .uri()
            .query()
            .expect("The query string should be exist at this location");
        match serde_qs::from_str(query) {
            Ok(v) => PollOutcome::Ready(v),
            Err(e) => PollOutcome::Abort(QueryError::Parsing(e).into()),
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

impl<F> FromData for Form<F>
where
    F: de::DeserializeOwned + 'static,
{
    type Error = QueryError;

    fn from_data(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_err(|_| QueryError::InvalidMediaType)?
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            serde_qs::from_bytes(&*body).map(Form).map_err(QueryError::Parsing)
        } else {
            Err(QueryError::InvalidMediaType)
        }
    }
}

/// All of error kinds when receiving/parsing the urlencoded data.
#[derive(Debug)]
pub enum QueryError {
    InvalidMediaType,
    Parsing(serde_qs::Error),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::QueryError::*;
        match *self {
            InvalidMediaType => f.write_str("The content type should be application/www-x-urlformencoded"),
            Parsing(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for QueryError {
    fn description(&self) -> &str {
        "failed to parse an urlencoded string"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            QueryError::Parsing(ref e) => Some(&*e),
            _ => None,
        }
    }
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
/// # extern crate finchers_endpoint;
/// # #[macro_use] extern crate serde;
/// # use finchers_http::query::{query, from_csv};
/// # use finchers_endpoint::EndpointExt;
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
