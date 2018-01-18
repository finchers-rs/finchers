//! `serde`-powered parsing of urlencoded strings, based on `serde_urlencoded`

extern crate serde;
extern crate serde_urlencoded;

use std::fmt;
use std::error::Error as StdError;
use std::marker::PhantomData;

use self::serde::de::DeserializeOwned;
use endpoint::{Endpoint, EndpointContext};
use http::{mime, FromBody, Request};

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
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let query_str = try_opt!(ctx.request().query());
        Some(self::serde_urlencoded::de::from_str(query_str).map_err(Into::into))
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
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match ctx.request().query() {
            Some(s) => Some(self::serde_urlencoded::de::from_str(s).map_err(Into::into)),
            None => Some(Err(QueriesError::missing())),
        }
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
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match ctx.request().query() {
            Some(query_str) => Some(
                self::serde_urlencoded::de::from_str(query_str)
                    .map(Some)
                    .map_err(Into::into),
            ),
            None => Some(Ok(None)),
        }
    }
}

/// An error from `Queries` and `QueriesOpt`
pub struct QueriesError<T: DeserializeOwned> {
    inner: Option<Error>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> QueriesError<T> {
    pub fn missing() -> Self {
        QueriesError {
            inner: None,
            _marker: PhantomData,
        }
    }

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

#[derive(Debug)]
pub struct Form<F: DeserializeOwned>(pub F);

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
