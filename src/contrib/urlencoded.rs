//! Support for parsing urlencoded queries and message body.
//!
//! Provided features:
//!
//! * `FromUrlEncoded` - Conversion from urlencoded string
//! * `Form` - Represents a type implemented `FromUrlEncoded`
//! * `queries` - Conversion the query string to a type implemented `FromUrlEncoded`

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

/// A wrapper struct which represents the contained type is parsed from `url-formencoded` body.
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

#[allow(missing_docs)]
pub fn queries<T: FromUrlEncoded>() -> Queries<T> {
    Queries {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Queries<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> Endpoint for Queries<T> {
    type Item = T;
    type Error = UrlDecodeError;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let query_str = try_opt!(ctx.request().query());
        let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
        Some(T::from_urlencoded(iter))
    }
}

#[allow(missing_docs)]
pub fn queries_opt<T: FromUrlEncoded>() -> QueriesOpt<T> {
    QueriesOpt {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct QueriesOpt<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromUrlEncoded> Endpoint for QueriesOpt<T> {
    type Item = Option<T>;
    type Error = UrlDecodeError;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match ctx.request().query() {
            Some(query_str) => {
                let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
                Some(T::from_urlencoded(iter).map(Some))
            }
            None => Some(Ok(None)),
        }
    }
}

/// The error type returned from `FromForm::from_form`.
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

pub use self::UrlDecodeError::*;

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
        "during parsing the urlencoded string"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            Other(ref e) => Some(&**e),
            _ => None,
        }
    }
}

impl IntoResponse for UrlDecodeError {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}

#[allow(missing_docs)]
pub mod serde {
    extern crate serde;
    extern crate serde_urlencoded;
    use std::marker::PhantomData;
    use self::serde::de::DeserializeOwned;
    use self::serde_urlencoded::de::Error;
    use endpoint::{Endpoint, EndpointContext};
    use http::{mime, FromBody, Request};

    #[allow(missing_docs)]
    pub fn queries<T: DeserializeOwned>() -> Queries<T> {
        Queries {
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    pub struct Queries<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T: DeserializeOwned> Endpoint for Queries<T> {
        type Item = T;
        type Error = Error;
        type Result = Result<Self::Item, Self::Error>;

        fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
            let query_str = try_opt!(ctx.request().query());
            Some(self::serde_urlencoded::de::from_str(query_str))
        }
    }

    #[allow(missing_docs)]
    pub fn queries_opt<T: DeserializeOwned>() -> QueriesOpt<T> {
        QueriesOpt {
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    pub struct QueriesOpt<T> {
        _marker: PhantomData<fn() -> T>,
    }

    impl<T: DeserializeOwned> Endpoint for QueriesOpt<T> {
        type Item = Option<T>;
        type Error = Error;
        type Result = Result<Self::Item, Self::Error>;

        fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
            match ctx.request().query() {
                Some(query_str) => Some(self::serde_urlencoded::de::from_str(query_str).map(Some)),
                None => Some(Ok(None)),
            }
        }
    }

    /// A wrapper struct which represents the contained type is parsed from `url-formencoded` body.
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
}
