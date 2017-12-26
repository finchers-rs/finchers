//! Support for parsing urlencoded queries and message body.
//!
//! # Example
//!
//! ```ignore
//! pub struct Foo {
//!     id: u32,
//!     name: String,
//! }
//!
//! impl FromForm for Foo {
//!     fn from_form(iter: FormPairs) -> Result<Self, FormError> {
//!         let (mut id, mut name) = (None, None);
//!         for (key, value) in iter {
//!             match key.as_str() {
//!                 "id" => {
//!                     if id.is_none() {
//!                         id = Some(value.parse()?);
//!                     } else {
//!                         return Err(FormError::duplicated_key("id"));
//!                     }
//!                 },
//!                 "name" => {
//!                     if name.is_none() {
//!                         name = Some(value.into_owned());
//!                     } else {
//!                         return Err(FormError::duplicated_key("name"));
//!                     }
//!                 },
//!                 key => return Err(FormError::missing_key(key)),
//!             }
//!         }
//!         Ok(Foo {
//!             id: id.ok_or_else(|| FormError::missing_key("id"))?,
//!             name: name.ok_or_else(|| FormError::missing_key("name"))?,
//!         })
//!     }
//! }
//! ```

#![allow(missing_docs)]

extern crate url;

use std::borrow::Cow;
use std::fmt;
use std::error::Error;
use http::{mime, FromBody, Request, StatusCode};
use responder::Responder;

/// A trait for parsing from `urlencoded` message body.
pub trait FromForm: Sized {
    /// Convert from the pairs of keys/values to itself.
    fn from_form<'a, I>(iter: I) -> Result<Self, FormError>
    where
        I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>;
}

/// A wrapper struct which represents the contained type is parsed from `url-formencoded` body.
#[derive(Debug)]
pub struct Form<F: FromForm>(pub F);

impl<F: FromForm> FromBody for Form<F> {
    type Error = FormError;

    fn validate(req: &Request) -> Result<(), Self::Error> {
        if !req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
        {
            return Err(FormError::BadRequest);
        }
        Ok(())
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        let iter = self::url::form_urlencoded::parse(&body);
        F::from_form(iter).map(Form)
    }
}

/// The error type returned from `FromForm::from_form`.
#[derive(Debug)]
pub enum FormError {
    BadRequest,
    /// The invalid key is exist.
    InvalidKey(Cow<'static, str>),
    /// The missing key is exist.
    MissingKey(Cow<'static, str>),
    /// The duplicated key is exist.
    DuplicatedKey(Cow<'static, str>),
    /// The other error
    Other(Box<Error + Send>),
}

pub use self::FormError::*;

impl FormError {
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

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BadRequest => f.write_str("bad request"),
            InvalidKey(ref key) => write!(f, "invalid key: \"{}\"", key),
            MissingKey(ref key) => write!(f, "missing key: \"{}\"", key),
            DuplicatedKey(ref key) => write!(f, "duplicated key: \"{}\"", key),
            Other(ref e) => e.fmt(f),
        }
    }
}

impl Error for FormError {
    fn description(&self) -> &str {
        "during parsing the urlencoded body"
    }
}

impl Responder for FormError {
    type Body = String;

    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
    }

    fn body(&mut self) -> Option<Self::Body> {
        Some(format!("{}: {}", self.description(), self))
    }
}
