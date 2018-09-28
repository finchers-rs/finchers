//! Components for parsing the incoming HTTP request.

mod body;
mod encoded;
mod global;
mod header;

pub use self::body::ReqBody;
pub use self::encoded::{EncodedStr, FromEncodedStr};
pub use self::header::FromHeaderValue;

#[doc(hidden)]
#[allow(deprecated)]
pub use self::global::with_get_cx;

// ====

use cookie::{Cookie, CookieJar};
use http;
use http::header::HeaderMap;
use http::Request;
use hyper::body::Body;
use mime::Mime;
use std::ops::Deref;

use error::{bad_request, Error};

/// The contextual information with an incoming HTTP request.
#[derive(Debug)]
pub struct Input {
    request: Request<ReqBody>,
    #[cfg_attr(feature = "lint", allow(clippy::option_option))]
    media_type: Option<Option<Mime>>,
    cookie_jar: Option<CookieJar>,
    response_headers: Option<HeaderMap>,
}

impl Input {
    pub(crate) fn new(request: Request<ReqBody>) -> Input {
        Input {
            request,
            media_type: None,
            cookie_jar: None,
            response_headers: None,
        }
    }

    /// Return a shared reference to the value of raw HTTP request without the message body.
    pub fn request(&self) -> &Request<ReqBody> {
        &self.request
    }

    /// Takes the instance of `RequestBody` from this value.
    #[inline]
    pub fn payload(&mut self) -> Option<Body> {
        self.request.body_mut().payload()
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
                        let raw_str = raw.to_str().map_err(bad_request)?;
                        let mime = raw_str.parse().map_err(bad_request)?;
                        Some(mime)
                    }
                    None => None,
                };
                Ok(self.media_type.get_or_insert(mime).as_ref())
            }
        }
    }

    /// Returns a `Cookies<'_>` or initialize the internal Cookie jar.
    pub fn cookies(&mut self) -> Result<&mut CookieJar, Error> {
        match self.cookie_jar {
            Some(ref mut jar) => Ok(jar),
            None => {
                let mut cookie_jar = CookieJar::new();

                for cookie in self.request.headers().get_all(http::header::COOKIE) {
                    let cookie_str = cookie.to_str().map_err(bad_request)?;
                    for s in cookie_str.split(';').map(|s| s.trim()) {
                        let cookie = Cookie::parse_encoded(s).map_err(bad_request)?.into_owned();
                        cookie_jar.add_original(cookie);
                    }
                }

                Ok(self.cookie_jar.get_or_insert(cookie_jar))
            }
        }
    }

    pub(crate) fn cookie_jar(&self) -> Option<&CookieJar> {
        self.cookie_jar.as_ref()
    }

    /// Returns a mutable reference to a `HeaderMap` which contains the entries of response headers.
    ///
    /// The values inserted in this header map are automatically added to the actual response.
    pub fn response_headers(&mut self) -> &mut HeaderMap {
        self.response_headers.get_or_insert_with(Default::default)
    }

    pub(crate) fn take_response_headers(&mut self) -> Option<HeaderMap> {
        self.response_headers.take()
    }
}

impl Deref for Input {
    type Target = Request<ReqBody>;

    fn deref(&self) -> &Self::Target {
        self.request()
    }
}
