//! Definitions and reexports of incoming HTTP requests

use futures::{Future, Poll, Stream};
use futures::future::{err, ok, AndThen, Flatten, FutureResult};
use hyper::{self, Headers, Method, Uri};
use hyper::header::{ContentType, Header};
use hyper::error::UriError;
use hyper::mime::{TEXT_PLAIN_UTF_8, APPLICATION_OCTET_STREAM};
use errors::{FinchersError, FinchersErrorKind, FinchersResult};


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
            body: self.inner,
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
    body: hyper::Body,
    buf: Option<Vec<u8>>,
}

impl IntoVec {
    fn poll_body(&mut self) -> Poll<Option<hyper::Chunk>, FinchersError> {
        self.body
            .poll()
            .map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
    }
}

impl Future for IntoVec {
    type Item = Vec<u8>;
    type Error = FinchersError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(item) = try_ready!(self.poll_body()) {
            if let Some(buf) = self.buf.as_mut() {
                buf.extend_from_slice(&item);
            }
        }

        let buf = self.buf.take().expect("The buffer has been already taken");
        Ok(buf.into())
    }
}


/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    #[allow(missing_docs)]
    type Error;

    /// A future returned from `from_body()`
    type Future: Future<Item = Self, Error = Self::Error>;

    /// Convert the content of `body` to its type
    fn from_body(body: Body, req: &Request) -> Self::Future;
}


impl FromBody for Vec<u8> {
    type Error = FinchersError;
    type Future = Flatten<FutureResult<IntoVec, FinchersError>>;

    fn from_body(body: Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_OCTET_STREAM => (),
            _ => return err(FinchersErrorKind::BadRequest.into()).flatten(),
        }

        ok(body.into_vec()).flatten()
    }
}

impl FromBody for String {
    type Error = FinchersError;
    type Future = Flatten<
        FutureResult<AndThen<IntoVec, FinchersResult<String>, fn(Vec<u8>) -> FinchersResult<String>>, FinchersError>,
    >;

    fn from_body(body: Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == TEXT_PLAIN_UTF_8 => (),
            _ => return err(FinchersErrorKind::BadRequest.into()).flatten(),
        }

        ok(body.into_vec().and_then(
            (|body| String::from_utf8(body).map_err(|_| FinchersErrorKind::BadRequest.into())) as
                fn(Vec<u8>) -> FinchersResult<String>,
        )).flatten()
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
