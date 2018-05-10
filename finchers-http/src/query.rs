//! Components for parsing the query string and urlencoded payload.

use bytes::Bytes;
use failure::{Fail, SyncFailure};
use http::StatusCode;
use serde::de::{self, DeserializeOwned, IntoDeserializer};
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use {mime, serde_qs};

use body::FromBody;
use finchers_core::endpoint::{Context, EncodedStr, Endpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, HttpError, Input, Poll, PollResult};

/// Create an endpoint which parse the query string in the HTTP request
/// to the value of `T`.
///
/// # Example
///
/// ```
/// # extern crate finchers_ext;
/// # extern crate finchers_http;
/// # #[macro_use] extern crate serde;
/// # use finchers_http::query::{query, from_csv, Serde};
/// # use finchers_ext::{EndpointExt, EndpointResultExt, EndpointOptionExt};
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
///     .map_ok(|param: Serde<Param>| format!("Received: {:?}", &*param))
///     .unwrap_ok();
/// # }
/// ```
pub fn query<T>() -> Query<T>
where
    T: FromQuery,
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
    T: FromQuery,
{
    type Output = Result<T, QueryError<T::Error>>;
    type Task = QueryTask<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(QueryTask { _marker: PhantomData })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct QueryTask<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Task for QueryTask<T>
where
    T: FromQuery,
{
    type Output = Result<T, QueryError<T::Error>>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        let ready = match cx.input().request().uri().query() {
            Some(query) => T::from_query(QueryItems::new(query)).map_err(|cause| QueryError::Parse { cause }),
            None => Err(QueryError::MissingQuery),
        };
        Poll::Ready(Ok(ready))
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

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if !input
            .media_type()
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
#[derive(Debug, Fail)]
pub enum QueryError<E> {
    #[fail(display = "The query string is not exist in the request")]
    MissingQuery,

    #[fail(display = "The content type must be application/www-x-urlformencoded")]
    InvalidMediaType,

    #[fail(display = "{}", cause)]
    Parse { cause: E },
}

impl<E: Fail> HttpError for QueryError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// Trait representing the transformation from a set of HTTP query.
pub trait FromQuery: Sized + 'static {
    /// The error type which will be returned from `from_query`.
    type Error;

    /// Perform transformation from `QueryItems` into `Self`.
    fn from_query(query: QueryItems) -> Result<Self, Self::Error>;
}

/// An iterator over the elements of query items.
#[derive(Debug)]
pub struct QueryItems<'a> {
    input: &'a [u8],
}

impl<'a> QueryItems<'a> {
    /// Create a new `QueryItems` from a slice of bytes.
    ///
    /// The input must be a valid HTTP query.
    pub fn new<S: AsRef<[u8]> + ?Sized>(input: &'a S) -> QueryItems<'a> {
        QueryItems { input: input.as_ref() }
    }

    /// Returns a slice of bytes which contains the remaining query items.
    #[inline(always)]
    pub fn as_slice(&self) -> &'a [u8] {
        self.input
    }
}

// FIXME: return an error if the input is invalid query sequence.
impl<'a> Iterator for QueryItems<'a> {
    type Item = (&'a EncodedStr, &'a EncodedStr);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.input.is_empty() {
                return None;
            }

            let mut s = self.input.splitn(2, |&b| b == b'&');
            let seq = s.next().unwrap();
            self.input = s.next().unwrap_or(&[]);
            if seq.is_empty() {
                continue;
            }

            let mut s = seq.splitn(2, |&b| b == b'=');
            let name = s.next().unwrap();
            let value = s.next().unwrap_or(&[]);
            break unsafe { Some((EncodedStr::new_unchecked(name), EncodedStr::new_unchecked(value))) };
        }
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
    de.deserialize_str(CSVSeqVisitor { _marker: PhantomData })
}
