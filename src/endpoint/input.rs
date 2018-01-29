use http::{header, Extensions, HeaderMap, Method, Request};
use http::request::Parts;
use hyper::{self, mime};
use core::{Body, BodyStream, RequestParts};

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Input {
    shared: RequestParts,
    body: Option<hyper::Body>,
    extensions: Extensions,
}

impl Input {
    #[allow(missing_docs)]
    pub fn from_request<R: Into<Request<hyper::Body>>>(request: R) -> Self {
        let (
            Parts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = request.into().into_parts();
        Input {
            shared: RequestParts::new(method, uri, version, headers),
            body: Some(body),
            extensions: extensions,
        }
    }

    pub fn parts(&self) -> &RequestParts {
        &self.shared
    }

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
    pub fn headers(&self) -> &HeaderMap {
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
    pub fn media_type(&self) -> Option<mime::Mime> {
        self.shared
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|s| s.to_str().ok().and_then(|s| s.parse().ok()))
    }

    #[allow(missing_docs)]
    pub fn shared_parts(&mut self) -> (RequestParts, Body) {
        let shared = self.shared.clone();
        let body = self.body().expect("cannot take the request body twice");
        (shared, body)
    }
}
