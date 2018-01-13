use hyper::{self, Headers, HttpVersion, Method, Uri};
use hyper::header::{self, Header};
use hyper::mime::Mime;
use super::Body;

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: HttpVersion,
    headers: Headers,
    body: Option<Body>,
}

impl From<hyper::Request> for Request {
    fn from(request: hyper::Request) -> Self {
        let (method, uri, version, headers, body) = request.deconstruct();
        Request {
            method,
            uri,
            version,
            headers,
            body: Some(body),
        }
    }
}

impl Request {
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

    #[allow(missing_docs)]
    pub fn body(&mut self) -> Option<Body> {
        self.body.take()
    }

    #[allow(missing_docs)]
    pub fn media_type(&self) -> Option<&Mime> {
        self.header().map(|&header::ContentType(ref m)| m)
    }
}
