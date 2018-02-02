use std::rc::Rc;
use http::{header, HeaderMap, Method, Uri, Version};
use mime;

/// Clonable, shared parts in the incoming HTTP request
#[derive(Debug, Clone)]
pub struct RequestParts {
    inner: Rc<Inner>,
}

#[derive(Debug)]
struct Inner {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
}

#[allow(missing_docs)]
impl RequestParts {
    pub(crate) fn new(method: Method, uri: Uri, version: Version, headers: HeaderMap) -> Self {
        RequestParts {
            inner: Rc::new(Inner {
                method,
                uri,
                version,
                headers,
            }),
        }
    }

    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    pub fn path(&self) -> &str {
        self.uri().path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri().query()
    }

    pub fn version(&self) -> &Version {
        &self.inner.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers
    }

    pub fn media_type(&self) -> Option<mime::Mime> {
        self.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|s| s.to_str().ok().and_then(|s| s.parse().ok()))
    }
}
