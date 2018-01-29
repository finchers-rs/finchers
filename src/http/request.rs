use std::rc::Rc;
use hyper::{self, Headers, HttpVersion, Method, Uri};
use hyper::header;
use hyper::mime::Mime;
use http_crate::{self, Extensions};
use super::{Body, BodyStream};

/// Clonable, shared parts in the incoming HTTP request
#[derive(Debug, Clone)]
pub struct RequestParts {
    inner: Rc<Inner>,
}

#[derive(Debug)]
struct Inner {
    method: Method,
    uri: Uri,
    version: HttpVersion,
    headers: Headers,
}

#[allow(missing_docs)]
impl RequestParts {
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    pub fn version(&self) -> &HttpVersion {
        &self.inner.version
    }

    pub fn headers(&self) -> &Headers {
        &self.inner.headers
    }
}

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Request {
    shared: RequestParts,
    body: Option<hyper::Body>,
    extensions: Extensions,
}

impl From<hyper::Request> for Request {
    fn from(request: hyper::Request) -> Self {
        let (method, uri, version, headers, body) = request.deconstruct();
        Request {
            shared: RequestParts {
                inner: Rc::new(Inner {
                    method,
                    uri,
                    version,
                    headers,
                }),
            },
            body: Some(body),
            extensions: Extensions::new(),
        }
    }
}

impl From<http_crate::Request<hyper::Body>> for Request {
    fn from(request: http_crate::Request<hyper::Body>) -> Self {
        let (parts, body) = request.into_parts();
        Request {
            shared: RequestParts {
                inner: Rc::new(Inner {
                    method: parts.method.into(),
                    uri: parts.uri.into(),
                    version: parts.version.into(),
                    headers: parts.headers.into(),
                }),
            },
            body: Some(body),
            extensions: parts.extensions,
        }
    }
}

impl Request {
    /// Return the reference of HTTP method
    pub fn method(&self) -> &Method {
        self.shared.method()
    }

    /// Return the path of HTTP request
    pub fn path(&self) -> &str {
        self.shared.uri().path()
    }

    /// Return the query part of HTTP request
    pub fn query(&self) -> Option<&str> {
        self.shared.uri().query()
    }

    /// Returns the shared reference of header map
    pub fn headers(&self) -> &Headers {
        self.shared.headers()
    }

    #[allow(missing_docs)]
    pub fn body(&mut self) -> Option<Body> {
        if let Some(stream) = self.body.take() {
            self.extensions.insert(Body::from(stream));
        }
        self.extensions.get::<Body>().cloned()
    }

    #[allow(missing_docs)]
    pub fn body_stream(&mut self) -> Option<BodyStream> {
        self.body.take().map(Into::into)
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
        self.shared
            .headers()
            .get()
            .map(|&header::ContentType(ref m)| m)
    }

    #[allow(missing_docs)]
    pub fn shared_parts(&mut self) -> (RequestParts, Body) {
        let shared = self.shared.clone();
        let body = self.body().expect("cannot take the request body twice");
        (shared, body)
    }
}
