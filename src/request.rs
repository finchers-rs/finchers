use hyper::{Method, Uri, Headers};
use hyper::header::Header;
pub use hyper::Body;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub headers: Headers,
    pub body: Option<Body>,
}

impl Request {
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri.query()
    }

    pub fn header<H: Header>(&self) -> Option<&H> {
        self.headers.get::<H>()
    }
}
