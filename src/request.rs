//! Definitions and reexports of incoming HTTP requests

use futures::{Async, Future, Poll, Stream};
use hyper::{self, Headers, Method, Uri};
use hyper::header::Header;
use hyper::error::UriError;
use errors::{FinchersError, FinchersErrorKind};


/// The value of incoming HTTP request, without the request body
#[derive(Debug)]
pub struct Request {
    pub(crate) method: Method,
    pub(crate) uri: Uri,
    pub(crate) headers: Headers,
}

impl Request {
    /// Create a new instance of `Request` from given HTTP method and URI
    pub fn new(method: Method, uri: &str) -> Result<Request, UriError> {
        Ok(Request {
            method,
            uri: uri.parse()?,
            headers: Default::default(),
        })
    }

    /// Return the reference of HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Return the path of HTTP request
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Return the query part of HTTP request
    pub fn query(&self) -> Option<&str> {
        self.uri.query()
    }

    /// Return the reference of the header of HTTP request
    pub fn header<H: Header>(&self) -> Option<&H> {
        self.headers.get::<H>()
    }
}


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

impl Body {
    /// Convert itself into the future of a `Vec<u8>`
    pub fn into_vec(self) -> IntoVec {
        IntoVec {
            body: self,
            buf: Some(Vec::new()),
        }
    }
}

impl Stream for Body {
    type Item = hyper::Chunk;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner
            .poll()
            .map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
    }
}

/// The type of a future returned from `Body::into_vec()`
#[derive(Debug)]
pub struct IntoVec {
    body: Body,
    buf: Option<Vec<u8>>,
}

impl Future for IntoVec {
    type Item = Vec<u8>;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match try_ready!(self.body.poll()) {
                Some(item) => {
                    let buf = self.buf
                        .as_mut()
                        .expect("The buffer has already been taken");
                    buf.extend_from_slice(&item);
                    continue;
                }
                None => {
                    let buf = self.buf.take().expect("cannot take the buffer twice");
                    break Ok(Async::Ready(buf));
                }
            }
        }
    }
}


/// reconstruct the raw incoming HTTP request, and return a pair of `Request` and `Body`
pub fn reconstruct(req: hyper::Request) -> (Request, Body) {
    let (method, uri, _version, headers, body) = req.deconstruct();
    let req = Request {
        method,
        uri,
        headers,
    };
    (req, body.into())
}
