//! `serde`-powered parsing of urlencoded strings, based on `serde_urlencoded`

extern crate serde;
extern crate serde_urlencoded;

use std::fmt;
use std::error::Error as StdError;
use std::marker::PhantomData;
use futures::future::{self, Future, FutureResult, IntoFuture};

use self::serde::de::DeserializeOwned;
use endpoint::{self, Endpoint, EndpointContext, EndpointResult};
use endpoint::body::BodyError;
use http::{self, mime, FromBody, Request};

pub use self::serde_urlencoded::de::Error;

#[allow(missing_docs)]
pub fn queries<T: DeserializeOwned>() -> Queries<T> {
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

impl<T: DeserializeOwned> Endpoint for Queries<T> {
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
pub struct QueriesResult<T: DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> EndpointResult for QueriesResult<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let result = self::serde_urlencoded::de::from_str(request.query().unwrap()).map_err(|e| Ok(e.into()));
        IntoFuture::into_future(result)
    }
}

#[allow(missing_docs)]
pub fn queries_req<T: DeserializeOwned>() -> QueriesRequired<T> {
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

impl<T: DeserializeOwned> Endpoint for QueriesRequired<T> {
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
pub struct QueriesRequiredResult<T: DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> EndpointResult for QueriesRequiredResult<T> {
    type Item = T;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let result = match request.query() {
            Some(s) => self::serde_urlencoded::de::from_str(s).map_err(|e| Ok(e.into())),
            None => Err(Ok(QueriesError::missing())),
        };
        IntoFuture::into_future(result)
    }
}

#[allow(missing_docs)]
pub fn queries_opt<T: DeserializeOwned>() -> QueriesOptional<T> {
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

impl<T: DeserializeOwned> Endpoint for QueriesOptional<T> {
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
pub struct QueriesOptionalResult<T: DeserializeOwned> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> EndpointResult for QueriesOptionalResult<T> {
    type Item = Option<T>;
    type Error = QueriesError<T>;
    type Future = FutureResult<Self::Item, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let result = match request.query() {
            Some(s) => self::serde_urlencoded::de::from_str(s)
                .map(Some)
                .map_err(|e| Ok(e.into())),
            None => Ok(None),
        };
        IntoFuture::into_future(result)
    }
}

/// An error from `Queries` and `QueriesOpt`
pub struct QueriesError<T: DeserializeOwned> {
    inner: Option<Error>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> QueriesError<T> {
    fn missing() -> Self {
        QueriesError {
            inner: None,
            _marker: PhantomData,
        }
    }

    /// Returns the internal value if possible
    pub fn inner(&self) -> Option<&Error> {
        self.inner.as_ref()
    }
}

impl<T: DeserializeOwned> From<Error> for QueriesError<T> {
    fn from(inner: Error) -> Self {
        QueriesError {
            inner: Some(inner),
            _marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned> fmt::Debug for QueriesError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("QueriesError").field(&self.inner).finish()
    }
}

impl<T: DeserializeOwned> fmt::Display for QueriesError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Some(ref e) => e.fmt(f),
            None => f.write_str("missing query string"),
        }
    }
}

impl<T: DeserializeOwned> StdError for QueriesError<T> {
    fn description(&self) -> &str {
        "failed to parse an urlencoded string"
    }

    fn cause(&self) -> Option<&StdError> {
        self.inner.as_ref().and_then(|e| e.cause())
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

impl<F: DeserializeOwned> FromBody for Form<F> {
    type Error = Error;

    fn validate(req: &Request) -> bool {
        req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        self::serde_urlencoded::from_bytes(&body).map(Form)
    }
}

#[allow(missing_docs)]
pub fn form_body<T: DeserializeOwned>() -> FormBody<T> {
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

impl<T: DeserializeOwned> Endpoint for FormBody<T> {
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

impl<T: DeserializeOwned> EndpointResult for FormBodyResult<T> {
    type Item = T;
    type Error = BodyError<Form<T>>;
    type Future = future::Map<endpoint::body::BodyFuture<Form<T>>, fn(Form<T>) -> T>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        self.inner.into_future(request).map(|Form(body)| body)
    }
}
