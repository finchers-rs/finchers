//! Components for parsing the incoming HTTP request.

mod encoded;
mod header;

pub use self::encoded::{EncodedStr, FromEncodedStr};
pub use self::header::FromHeaderValue;

// ====

use http;
use http::header::HeaderMap;
use http::Request;
use mime::Mime;

use crate::error::{BadRequest, Error};

/// The contextual information with an incoming HTTP request.
#[derive(Debug)]
pub struct Input<Bd> {
    request: Request<()>,
    body: Option<Bd>,
    #[allow(clippy::option_option)]
    media_type: Option<Option<Mime>>,
    pub(crate) response_headers: Option<HeaderMap>,
}

impl<Bd> Input<Bd> {
    pub(crate) fn new(request: Request<Bd>) -> Self {
        let (parts, body) = request.into_parts();
        Input {
            request: Request::from_parts(parts, ()),
            body: Some(body),
            media_type: None,
            response_headers: None,
        }
    }

    pub(crate) fn request(&self) -> &Request<()> {
        &self.request
    }

    /// Returns a reference to the HTTP method of the request.
    pub fn method(&self) -> &http::Method {
        self.request.method()
    }

    /// Returns a reference to the URI of the request.
    pub fn uri(&self) -> &http::Uri {
        self.request.uri()
    }

    /// Returns the HTTP version of the request.
    pub fn version(&self) -> http::Version {
        self.request.version()
    }

    /// Returns a reference to the header map in the request.
    pub fn headers(&self) -> &http::HeaderMap {
        self.request.headers()
    }

    /// Returns a reference to the extension map which contains
    /// extra information about the request.
    pub fn extensions(&self) -> &http::Extensions {
        self.request.extensions()
    }

    /// Returns a mutable reference to the message body in the request.
    pub fn body(&mut self) -> &mut Option<Bd> {
        &mut self.body
    }

    /// Attempts to get the entry of `Content-type` and parse its value.
    ///
    /// The result of this method is cached and it will return the reference to the cached value
    /// on subsequent calls.
    pub fn content_type(&mut self) -> Result<Option<&Mime>, Error> {
        match self.media_type {
            Some(ref m) => Ok(m.as_ref()),
            None => {
                let mime = match self.request.headers().get(http::header::CONTENT_TYPE) {
                    Some(raw) => {
                        let raw_str = raw.to_str().map_err(BadRequest::from)?;
                        let mime = raw_str.parse().map_err(BadRequest::from)?;
                        Some(mime)
                    }
                    None => None,
                };
                Ok(self.media_type.get_or_insert(mime).as_ref())
            }
        }
    }

    /// Returns a mutable reference to a `HeaderMap` which contains the entries of response headers.
    ///
    /// The values inserted in this header map are automatically added to the actual response.
    pub fn response_headers(&mut self) -> &mut HeaderMap {
        self.response_headers.get_or_insert_with(Default::default)
    }
}
