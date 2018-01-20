//! A simple support for parsing of urlencoded strings, using the feature in `url` crate.

extern crate url;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use futures::future::{self, Future, FutureResult, IntoFuture};

use endpoint::{self, Endpoint, EndpointContext, EndpointResult};
use endpoint::body::BodyError;
use errors::StdErrorResponseBuilder;
use http::{self, mime, FromBody, IntoResponse, Request, Response};

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
    type Result = QueriesResult<T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if ctx.request().query().is_some() {
            Some(QueriesResult {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesResult<T: FromUrlEncoded> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> EndpointResult for QueriesResult<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let query_str = request.query().unwrap();
        let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
        IntoFuture::into_future(T::from_urlencoded(iter).map_err(|e| Ok(e.into())))
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
    type Result = QueriesRequiredResult<T>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(QueriesRequiredResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesRequiredResult<T: FromUrlEncoded> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> EndpointResult for QueriesRequiredResult<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let result = match request.query() {
            Some(s) => {
                let iter = self::url::form_urlencoded::parse(s.as_bytes());
                T::from_urlencoded(iter).map_err(|e| Ok(e.into()))
            }
            None => Err(Ok(QueriesError::missing())),
        };
        IntoFuture::into_future(result)
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
    type Result = QueriesOptionalResult<T>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(QueriesOptionalResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct QueriesOptionalResult<T: FromUrlEncoded> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> EndpointResult for QueriesOptionalResult<T> {
    type Item = Option<T>;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let result = match request.query() {
            Some(s) => {
                let iter = self::url::form_urlencoded::parse(s.as_bytes());
                T::from_urlencoded(iter).map(Some).map_err(|e| Ok(e.into()))
            }
            None => Ok(None),
        };
        IntoFuture::into_future(result)
    }
}

/// An error from `Queries` and `QueriesOpt`
pub struct QueriesError<T: FromUrlEncoded> {
    inner: Option<UrlDecodeError>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> QueriesError<T> {
    #[allow(missing_docs)]
    fn missing() -> Self {
        QueriesError {
            inner: None,
            _marker: PhantomData,
        }
    }

    /// Returns the reference of internal value, if possible.
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

#[allow(missing_docs)]
pub fn form_body<T: FromUrlEncoded>() -> FormBody<T> {
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

impl<T: FromUrlEncoded> Endpoint for FormBody<T> {
    type Item = T;
    type Error = BodyError<Form<T>>;
    type Result = FormBodyResult<T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        Some(FormBodyResult {
            inner: try_opt!(self.inner.apply(ctx)),
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct FormBodyResult<T> {
    inner: endpoint::body::BodyResult<Form<T>>,
}

impl<T: FromUrlEncoded> EndpointResult for FormBodyResult<T> {
    type Item = T;
    type Error = BodyError<Form<T>>;
    type Future = future::Map<endpoint::body::BodyFuture<Form<T>>, fn(Form<T>) -> T>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        self.inner.into_future(request).map(|Form(body)| body)
    }
}
