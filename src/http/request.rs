use std::ops::{Deref, DerefMut};
use hyper::{self, Body};
use http_crate;

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Request {
    inner: http_crate::Request<Option<Body>>,
}

impl From<hyper::Request> for Request {
    fn from(request: hyper::Request) -> Self {
        let inner = http_crate::Request::from(request).map(Some);
        Request { inner }
    }
}

impl From<http_crate::Request<Body>> for Request {
    fn from(request: http_crate::Request<Body>) -> Self {
        Request {
            inner: request.map(Some),
        }
    }
}

impl Deref for Request {
    type Target = http_crate::Request<Option<Body>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Request {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
