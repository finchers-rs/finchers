use std::rc::Rc;
use hyper::{Headers, HttpVersion, Method, Uri};

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
    pub(crate) fn new(method: Method, uri: Uri, version: HttpVersion, headers: Headers) -> Self {
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

    pub fn version(&self) -> &HttpVersion {
        &self.inner.version
    }

    pub fn headers(&self) -> &Headers {
        &self.inner.headers
    }
}
