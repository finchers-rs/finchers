//! Support for parsing urlencoded queries or message body, based on `serde_qs`
//!
//! Provided endpoins/utilities are as follows:
//!
//! * `Queries<T>` - Parses the query string in incoming request to the value of `T`, otherwise skips the current route.
//! * `QueriesRequired<T>` - Similar to `Queries`, but always matches and returns an error if the query is missing.
//! * `QueriesOptional<T>` - Similar to `Queries`, but always matches and returns an `Option<T>`.
//! * `Form<T>` - Represents a type deserialized from an urlencoded request body.
//!
//! # Examples
//!
//! ```ignore
//! #[derive(Debug, Deserialize)]
//! struct Param {
//!     name: String,
//!     required: bool,
//! }
//!
//! let endpoint = queries_req::<Param>();
//! ```
//!
//! ```ignore
//! let endpoint = get(queries().map_err(Into::into))
//!     .or(post(body().map(|Form(body)| body)).map_err(Into::into))
//!     .and_then(|param| { ... });
//! ```

//#![doc(html_root_url = "https://docs.rs/finchers/0.10.1")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate finchers_core;
extern crate futures;
extern crate mime;
extern crate serde;
extern crate serde_qs;

use std::{error, fmt};
use std::marker::PhantomData;
use std::iter::FromIterator;
use serde::de::{self, IntoDeserializer};
use futures::{Future, Poll};

use finchers_core::endpoint::{self, Context, Endpoint};
use finchers_core::error::{BadRequest, Error as FinchersError};
use finchers_core::request::{with_input, Bytes, FromBody, Input};

#[allow(missing_docs)]
pub fn queries<T: de::DeserializeOwned>() -> Queries<T> {
    Queries {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Queries<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Queries<T> {}

impl<T> Clone for Queries<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Queries<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Queries").finish()
    }
}

impl<T: de::DeserializeOwned> Endpoint for Queries<T> {
    type Item = T;
    type Future = QueriesFuture<T>;

    fn apply(&self, input: &Input, _: &mut Context) -> Option<Self::Future> {
        if input.query().is_some() {
            Some(QueriesFuture {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesFuture<T: de::DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: de::DeserializeOwned> Future for QueriesFuture<T> {
    type Item = T;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = with_input(|input| serde_qs::from_str::<T>(input.query().unwrap()).map_err(Error::Parsing));
        result
            .map(Into::into)
            .map_err(|e| BadRequest::new(e).into())
    }
}

#[allow(missing_docs)]
pub fn queries_req<T: de::DeserializeOwned>() -> QueriesRequired<T> {
    QueriesRequired {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct QueriesRequired<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for QueriesRequired<T> {}

impl<T> Clone for QueriesRequired<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for QueriesRequired<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("QueriesRequired").finish()
    }
}

impl<T: de::DeserializeOwned> Endpoint for QueriesRequired<T> {
    type Item = T;
    type Future = QueriesRequiredFuture<T>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(QueriesRequiredFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesRequiredFuture<T: de::DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: de::DeserializeOwned> Future for QueriesRequiredFuture<T> {
    type Item = T;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = with_input(|input| match input.query() {
            Some(s) => self::serde_qs::from_str::<T>(s).map_err(Error::Parsing),
            None => Err(Error::MissingQuery),
        });
        result
            .map(Into::into)
            .map_err(|e| BadRequest::new(e).into())
    }
}

#[allow(missing_docs)]
pub fn queries_opt<T: de::DeserializeOwned>() -> QueriesOptional<T> {
    QueriesOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct QueriesOptional<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for QueriesOptional<T> {}

impl<T> Clone for QueriesOptional<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for QueriesOptional<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("QueriesOptional").finish()
    }
}

impl<T: de::DeserializeOwned> Endpoint for QueriesOptional<T> {
    type Item = Option<T>;
    type Future = QueriesOptionalFuture<T>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(QueriesOptionalFuture {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesOptionalFuture<T: de::DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: de::DeserializeOwned> Future for QueriesOptionalFuture<T> {
    type Item = Option<T>;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = with_input(|input| match input.query() {
            Some(s) => match serde_qs::from_str(s) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(BadRequest::new(Error::Parsing(e)).into()),
            },
            None => Ok(None),
        });
        result.map(Into::into)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Form<F>(pub F);

impl<F> From<F> for Form<F> {
    #[inline]
    fn from(inner: F) -> Self {
        Form(inner)
    }
}

impl<F> ::std::ops::Deref for Form<F> {
    type Target = F;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> ::std::ops::DerefMut for Form<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: de::DeserializeOwned + 'static> FromBody for Form<F> {
    type Error = Error;

    fn from_body(body: Bytes, input: &Input) -> Result<Self, Self::Error> {
        if input
            .media_type()
            .map_or(true, |m| m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            serde_qs::from_bytes(&*body).map(Form).map_err(Into::into)
        } else {
            Err(Error::InvalidMediaType)
        }
    }
}

#[allow(missing_docs)]
pub fn form_body<T: de::DeserializeOwned + 'static>() -> FormBody<T> {
    FormBody {
        inner: endpoint::body::body(),
    }
}

#[allow(missing_docs)]
pub struct FormBody<T> {
    inner: endpoint::body::Body<Form<T>>,
}

impl<T> Copy for FormBody<T> {}

impl<T> Clone for FormBody<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for FormBody<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FormBody").field(&self.inner).finish()
    }
}

impl<T: de::DeserializeOwned + 'static> Endpoint for FormBody<T> {
    type Item = T;
    type Future = FormBodyFuture<T>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        Some(FormBodyFuture {
            inner: match self.inner.apply(input, ctx) {
                Some(inner) => inner,
                None => return None,
            },
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct FormBodyFuture<T> {
    inner: endpoint::body::BodyFuture<Form<T>>,
}

impl<T: de::DeserializeOwned + 'static> Future for FormBodyFuture<T> {
    type Item = T;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll().map(|async| async.map(|Form(body)| body))
    }
}

/// An error from `Queries` and `QueriesOpt`
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error {
    MissingQuery,
    InvalidMediaType,
    Parsing(self::serde_qs::Error),
}

impl From<self::serde_qs::Error> for Error {
    fn from(err: self::serde_qs::Error) -> Self {
        Error::Parsing(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MissingQuery => f.write_str("missing query string"),
            Error::InvalidMediaType => f.write_str("The content type should be application/www-x-urlformencoded"),
            Error::Parsing(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "failed to parse an urlencoded string"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Parsing(ref e) => Some(&*e),
            _ => None,
        }
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

/// Deserialize a sequece from a comma-separated string
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
