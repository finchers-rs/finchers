use hyper::{self, Headers, HttpVersion, Method, Uri};
use hyper::header;
use hyper::mime::Mime;
use http_crate::{self, Extensions};

use super::Body;

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: HttpVersion,
    headers: Headers,
    body: Option<Body>,
    extensions: Extensions,
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
            extensions: Extensions::new(),
        }
    }
}

impl From<http_crate::Request<Body>> for Request {
    fn from(request: http_crate::Request<Body>) -> Self {
        let (parts, body) = request.into_parts();
        Request {
            method: parts.method.into(),
            uri: parts.uri.into(),
            version: parts.version.into(),
            headers: parts.headers.into(),
            body: Some(body),
            extensions: parts.extensions,
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

    /// Returns the shared reference of header map
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns the mutable reference of header map
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    #[allow(missing_docs)]
    pub fn body(&mut self) -> Option<Body> {
        self.body.take()
    }

    #[allow(missing_docs)]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[allow(missing_docs)]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    #[allow(missing_docs)]
    pub fn media_type(&self) -> Option<&Mime> {
        self.headers.get().map(|&header::ContentType(ref m)| m)
    }
}
