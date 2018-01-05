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
use http::{mime, FromBody, Request, StatusCode};
use responder::ErrorResponder;

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

    fn validate(req: &Request) -> Result<(), Self::Error> {
        if !req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            return Err(UrlDecodeError::InvalidMediaType);
        }
        Ok(())
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
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let query_str = match ctx.request().query() {
            Some(s) => s,
            None => return Some(Err(UrlDecodeError::EmptyQuery)),
        };
        let iter = self::url::form_urlencoded::parse(query_str.as_bytes());
        Some(T::from_urlencoded(iter))
    }
}

/// The error type returned from `FromForm::from_form`.
#[derive(Debug)]
pub enum UrlDecodeError {
    /// The query string is empty.
    EmptyQuery,
    /// The value of `Content-type` is not `application/www-x-form-urlencoded`.
    InvalidMediaType,
    /// The invalid key is exist.
    InvalidKey(Cow<'static, str>),
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
            EmptyQuery => f.write_str("empty query"),
            InvalidMediaType => f.write_str("The media type is invalid"),
            InvalidKey(ref key) => write!(f, "invalid key: \"{}\"", key),
            MissingKey(ref key) => write!(f, "missing key: \"{}\"", key),
            DuplicatedKey(ref key) => write!(f, "duplicated key: \"{}\"", key),
            Other(ref e) => e.fmt(f),
        }
    }
}

impl Error for UrlDecodeError {
    fn description(&self) -> &str {
        "during parsing the urlencoded body"
    }
}

impl ErrorResponder for UrlDecodeError {
    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
    }
}
