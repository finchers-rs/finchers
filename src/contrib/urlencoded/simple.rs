//! A simple support for parsing of urlencoded strings, using the feature in `url` crate.

extern crate url;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use errors::StdErrorResponseBuilder;
use http::{mime, FromBody, IntoResponse, Request, Response};

pub use self::url::form_urlencoded::Parse;

/// A trait for parsing from `urlencoded` message body.
pub trait FromUrlEncoded: Sized {
    /// Convert from the pairs of keys/values to itself.
    fn from_urlencoded(iter: Parse) -> Result<Self, UrlDecodeError>;
}

/// A type represents errors during parsing URL-encoded strings
#[derive(Debug)]
pub enum UrlDecodeError {
    /// The invalid key is exist.
    InvalidKey(Cow<'static, str>),
    /// The value is invalid.
    InvalidValue(Cow<'static, str>, Cow<'static, str>),
    /// The missing key is exist.
    MissingKey(Cow<'static, str>),
    /// The duplicated key is exist.
    DuplicatedKey(Cow<'static, str>),
    /// The other error
    Other(Box<Error + Send>),
}

use self::UrlDecodeError::*;

impl UrlDecodeError {
    #[allow(missing_docs)]
    pub fn invalid_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        InvalidKey(key.into())
    }

    #[allow(missing_docs)]
    pub fn invalid_value<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(key: K, value: V) -> Self {
        InvalidValue(key.into(), value.into())
    }

    #[allow(missing_docs)]
    pub fn missing_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        MissingKey(key.into())
    }

    #[allow(missing_docs)]
    pub fn duplicated_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        DuplicatedKey(key.into())
    }

    #[allow(missing_docs)]
    pub fn other<E: Error + Send + 'static>(err: E) -> Self {
        Other(Box::new(err))
    }
}

impl fmt::Display for UrlDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidKey(ref key) => write!(f, "invalid key: \"{}\"", key),
            InvalidValue(ref key, ref value) => write!(f, "invalid value: \"{}\" => \"{}\"", key, value),
            MissingKey(ref key) => write!(f, "missing key: \"{}\"", key),
            DuplicatedKey(ref key) => write!(f, "duplicated key: \"{}\"", key),
            Other(ref e) => e.fmt(f),
        }
    }
}

impl Error for UrlDecodeError {
    fn description(&self) -> &str {
        "during parsing urlencoded string"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            Other(ref e) => Some(&**e),
            _ => None,
        }
    }
}

impl FromUrlEncoded for HashMap<String, Vec<String>> {
    fn from_urlencoded(iter: Parse) -> Result<Self, UrlDecodeError> {
        let mut queries = HashMap::new();
        for (key, value) in iter {
            queries
                .entry(key.into_owned())
                .or_insert_with(|| vec![])
                .push(value.into_owned());
        }
        Ok(queries)
    }
}

#[allow(missing_docs)]
pub fn queries<T: FromUrlEncoded>() -> Queries<T> {
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

impl<T: FromUrlEncoded> Endpoint for Queries<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let query_str = try_opt!(ctx.request().query());
        let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
        Some(T::from_urlencoded(iter).map_err(Into::into))
    }
}

#[allow(missing_docs)]
pub fn queries_req<T: FromUrlEncoded>() -> QueriesRequired<T> {
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

impl<T: FromUrlEncoded> Endpoint for QueriesRequired<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match ctx.request().query() {
            Some(s) => {
                let iter = self::url::form_urlencoded::parse(s.as_bytes());
                Some(T::from_urlencoded(iter).map_err(Into::into))
            }
            None => Some(Err(QueriesError::missing())),
        }
    }
}

#[allow(missing_docs)]
pub fn queries_opt<T: FromUrlEncoded>() -> QueriesOptional<T> {
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

impl<T: FromUrlEncoded> Endpoint for QueriesOptional<T> {
    type Item = Option<T>;
    type Error = QueriesError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match ctx.request().query() {
            Some(query_str) => {
                let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
                Some(T::from_urlencoded(iter).map(Some).map_err(Into::into))
            }
            None => Some(Ok(None)),
        }
    }
}

/// An error from `Queries` and `QueriesOpt`
pub struct QueriesError<T: FromUrlEncoded> {
    inner: Option<UrlDecodeError>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> QueriesError<T> {
    pub fn missing() -> Self {
        QueriesError {
            inner: None,
            _marker: PhantomData,
        }
    }

    pub fn inner(&self) -> Option<&UrlDecodeError> {
        self.inner.as_ref()
    }
}

impl<T: FromUrlEncoded> From<UrlDecodeError> for QueriesError<T> {
    fn from(inner: UrlDecodeError) -> Self {
        QueriesError {
            inner: Some(inner),
            _marker: PhantomData,
        }
    }
}

impl<T: FromUrlEncoded> fmt::Debug for QueriesError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("QueriesError")
            .field("kind", &self.inner)
            .finish()
    }
}

impl<T: FromUrlEncoded> fmt::Display for QueriesError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Some(ref e) => e.fmt(f),
            None => f.write_str("missing query string"),
        }
    }
}

impl<T: FromUrlEncoded> Error for QueriesError<T> {
    fn description(&self) -> &str {
        "failed to parse a query string"
    }

    fn cause(&self) -> Option<&Error> {
        self.inner.as_ref().and_then(|e| e.cause())
    }
}

impl<T: FromUrlEncoded> IntoResponse for QueriesError<T> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}

#[derive(Debug)]
pub struct Form<F: FromUrlEncoded>(pub F);

impl<F: FromUrlEncoded> FromBody for Form<F> {
    type Error = UrlDecodeError;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        let iter = self::url::form_urlencoded::parse(&body);
        F::from_urlencoded(iter).map(Form)
    }
}
