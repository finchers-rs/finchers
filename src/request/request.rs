use hyper::{Headers, Method, Uri};
use hyper::header::{self, Header};
use hyper::mime::Mime;
use hyper::error::UriError;


/// The value of incoming HTTP request, without the request body
#[derive(Debug)]
pub struct Request {
    pub(crate) method: Method,
    pub(crate) uri: Uri,
    pub(crate) headers: Headers,
}

impl Request {
    /// Create a new instance of `Request` from given HTTP method and URI
    pub fn new(method: Method, uri: &str) -> Result<Request, UriError> {
        Ok(Request {
            method,
            uri: uri.parse()?,
            headers: Default::default(),
        })
    }

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

    /// Return the reference of the header of HTTP request
    pub fn header<H: Header>(&self) -> Option<&H> {
        self.headers.get::<H>()
    }

    #[allow(missing_docs)]
    pub fn media_type(&self) -> Option<&Mime> {
        self.header().map(|&header::ContentType(ref m)| m)
    }
}
