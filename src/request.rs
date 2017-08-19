use hyper::{self, Method, Uri, Headers};
use hyper::header::Header;
use hyper::error::UriError;
pub use hyper::Body;

#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    headers: Headers,
    body: Option<Body>,
}

impl Request {
    pub fn new(method: Method, uri: &str) -> Result<Request, UriError> {
        Ok(Request {
            method,
            uri: uri.parse()?,
            headers: Default::default(),
            body: Some(Body::default()),
        })
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

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

pub fn reconstruct(req: hyper::Request) -> Request {
    let (method, uri, _version, headers, body) = req.deconstruct();
    let req = Request {
        method,
        uri,
        headers,
        body: Some(body),
    };
    req
}
