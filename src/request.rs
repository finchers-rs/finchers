use hyper::{Method, Uri, Headers};
pub use hyper::Body;

pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub headers: Headers,
}

impl Request {
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri.query()
    }
}
