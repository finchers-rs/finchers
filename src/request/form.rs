#![allow(missing_docs)]

use std::{error, fmt};
use hyper::mime;
use url::form_urlencoded::{self, Parse};
use super::{FromBody, Request};


pub trait FromForm: Sized {
    fn from_form(iter: Parse) -> Result<Self, FormParseError>;
}


#[derive(Debug)]
pub struct Form<F>(pub F);

impl<F: FromForm> FromBody for Form<F> {
    type Error = FormParseError;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(false, |m| *m == mime::APPLICATION_WWW_FORM_URLENCODED)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        let iter = form_urlencoded::parse(&body);
        F::from_form(iter).map(Form)
    }
}


#[derive(Debug)]
pub enum FormParseError {
    InvalidKey(String),
    MissingKey(String),
    DuplicatedKey(String),
}

impl fmt::Display for FormParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FormParseError::InvalidKey(ref key) => write!(f, "invalid key: \"{}\"", key),
            FormParseError::MissingKey(ref key) => write!(f, "missing key: \"{}\"", key),
            FormParseError::DuplicatedKey(ref key) => write!(f, "duplicated key: \"{}\"", key),
        }
    }
}

impl error::Error for FormParseError {
    fn description(&self) -> &str {
        "failed to parse an urlencoded body"
    }
}
