//! Components for parsing the incoming HTTP request.

mod body;
mod encoded;
mod header;

pub use self::body::{Payload, ReqBody};
pub use self::encoded::{EncodedStr, FromEncodedStr};
pub use self::header::FromHeaderValue;

// ====

use cookie::{Cookie, CookieJar};
use futures::Future;
use http;
use http::header::{HeaderMap, HeaderValue};
use http::Request;
use http::{Response, StatusCode};
use mime::Mime;

use error::{bad_request, Error};

type Task = Box<dyn Future<Item = (), Error = ()> + Send + 'static>;

/// The contextual information with an incoming HTTP request.
#[derive(Debug)]
pub struct Input {
    request: Request<ReqBody>,
    #[cfg_attr(feature = "cargo-clippy", allow(option_option))]
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

    /// Returns a reference to the message body in the request.
    pub fn body(&self) -> &ReqBody {
        self.request.body()
    }

    /// Returns a mutable reference to the message body in the request.
    pub fn body_mut(&mut self) -> &mut ReqBody {
        self.request.body_mut()
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

    /// Returns a mutable reference to a `HeaderMap` which contains the entries of response headers.
    ///
    /// The values inserted in this header map are automatically added to the actual response.
    pub fn response_headers(&mut self) -> &mut HeaderMap {
        self.response_headers.get_or_insert_with(Default::default)
    }

    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    pub(crate) fn finalize<T>(
        self,
        output: Result<Response<T>, Error>,
    ) -> (Response<Result<Option<T>, Error>>, Option<Task>) {
        let (_parts, body) = self.request.into_parts();
        let mut upgraded_opt = None;

        let mut response = match output {
            Ok(mut response) => match body.into_upgraded() {
                Some(upgraded) => {
                    upgraded_opt = Some(upgraded);

                    // Forcibly rewrite the status code and response body.
                    // Since these operations are automaically done by Hyper,
                    // they are substantially unnecessary.
                    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
                    response.map(|_bd| Ok(None))
                }
                None => response.map(|bd| Ok(Some(bd))),
            },
            Err(err) => err.into_response().map(Err),
        };

        if let Some(ref jar) = self.cookie_jar {
            for cookie in jar.delta() {
                let val = HeaderValue::from_str(&cookie.encoded().to_string()).unwrap();
                response.headers_mut().append(http::header::SET_COOKIE, val);
            }
        }

        if let Some(headers) = self.response_headers {
            response.headers_mut().extend(headers);
        }

        (response, upgraded_opt)
    }
}
