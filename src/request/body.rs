use std::marker::PhantomData;
use std::string::FromUtf8Error;
use futures::{Future, Poll, Stream};
use hyper;
use hyper::header::ContentType;
use hyper::mime::{TEXT_PLAIN_UTF_8, APPLICATION_OCTET_STREAM};

use super::request::Request;
use util::NoReturn;

/// The instance of request body.
#[derive(Default, Debug)]
pub struct Body {
    inner: hyper::Body,
}

impl From<hyper::Body> for Body {
    fn from(body: hyper::Body) -> Self {
        Self { inner: body }
    }
}

impl<T: FromBody> Into<ParseBody<T>> for Body {
    fn into(self) -> ParseBody<T> {
        ParseBody {
            body: self.inner,
            buf: Some(Vec::new()),
            _marker: PhantomData,
        }
    }
}


/// The type of a future returned from `Body::into_vec()`
#[derive(Debug)]
pub struct ParseBody<T> {
    body: hyper::Body,
    buf: Option<Vec<u8>>,
    _marker: PhantomData<T>,
}

impl<T: FromBody> Future for ParseBody<T> {
    type Item = T;
    type Error = ParseBodyError<T::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(item) = try_ready!(self.body.poll()) {
            if let Some(buf) = self.buf.as_mut() {
                buf.extend_from_slice(&item);
            }
        }

        let buf = self.buf.take().expect("The buffer has been already taken");
        T::from_body(buf)
            .map(Into::into)
            .map_err(ParseBodyError::Parse)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum ParseBodyError<E> {
    Hyper(hyper::Error),
    Parse(E),
}

impl<T> From<hyper::Error> for ParseBodyError<T> {
    fn from(err: hyper::Error) -> Self {
        ParseBodyError::Hyper(err)
    }
}


/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    #[allow(missing_docs)]
    type Error;

    #[allow(missing_docs)]
    fn check_request(req: &Request) -> bool;

    /// Convert the content of `body` to its type
    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error>;
}


impl FromBody for Vec<u8> {
    type Error = NoReturn;

    fn check_request(req: &Request) -> bool {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_OCTET_STREAM => true,
            _ => false,
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn check_request(req: &Request) -> bool {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == TEXT_PLAIN_UTF_8 => true,
            _ => false,
        }
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        String::from_utf8(body)
    }
}
