use std::borrow::Cow;
use std::{error, fmt};
use hyper::mime;
use url::form_urlencoded::{self, Parse};
use super::{FromBody, Request};











































pub trait FromForm: Sized {
    /// The error type during `from_form`.
    type Error: error::Error;

    /// Convert the pairs of parsed message body to this type.
    fn from_form(iter: FormPairs) -> Result<Self, FormError<Self::Error>>;
}



#[derive(Debug)]
pub struct Form<F: FromForm>(pub F);

impl<F: FromForm> FromBody for Form<F> {
    type Error = FormError<F::Error>;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(true, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        let iter = form_urlencoded::parse(&body);
        F::from_form(FormPairs(iter)).map(Form)
    }
}



#[derive(Debug)]
pub enum FormError<E> {
    /// The invalid key is exist.
    InvalidKey(Cow<'static, str>),
    /// The missing key is exist.
    MissingKey(Cow<'static, str>),
    /// The duplicated key is exist.
    DuplicatedKey(Cow<'static, str>),
    /// The other error
    Other(E),
}

pub use self::FormError::*;

impl<E> FormError<E> {
    pub fn invalid_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        InvalidKey(key.into())
    }


    pub fn missing_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        MissingKey(key.into())
    }


    pub fn duplicated_key<S: Into<Cow<'static, str>>>(key: S) -> Self {
        DuplicatedKey(key.into())
    }
}

impl<E> From<E> for FormError<E> {
    fn from(e: E) -> Self {
        Other(e)
    }
}

impl<E: fmt::Display> fmt::Display for FormError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidKey(ref key) => write!(f, "invalid key: \"{}\"", key),
            MissingKey(ref key) => write!(f, "missing key: \"{}\"", key),
            DuplicatedKey(ref key) => write!(f, "duplicated key: \"{}\"", key),
            Other(ref e) => e.fmt(f),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> error::Error for FormError<E> {
    fn description(&self) -> &str {
        "during parsing the urlencoded body"
    }
}



#[derive(Debug, Copy, Clone)]
pub struct FormPairs<'a>(Parse<'a>);

impl<'a> Iterator for FormPairs<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
