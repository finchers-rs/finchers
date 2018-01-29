use std::rc::Rc;
use http::{HeaderMap, Method, Uri, Version};

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

    pub fn version(&self) -> &Version {
        &self.inner.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers
    }
}
