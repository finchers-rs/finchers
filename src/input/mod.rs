//! Components for parsing the incoming HTTP request.

pub mod body;
pub mod header;
pub mod query;

mod cursor;
mod encoded;
mod global;

pub use self::cursor::Cursor;
pub use self::encoded::{EncodedStr, FromEncodedStr};

pub use self::global::with_get_cx;
#[doc(hidden)]
pub use self::global::with_set_cx;

// ====

use failure::Fail;
use http;
use http::{Request, StatusCode};
use mime::{self, Mime};
use std::cell::UnsafeCell;
use std::marker::{PhantomData, Pinned};
use std::mem::PinMut;
use std::ops::Deref;

use self::body::{Payload, ReqBody};
use crate::error::HttpError;

/// The contextual information with an incoming HTTP request.
#[derive(Debug)]
pub struct Input {
    request: Request<ReqBody>,
    #[cfg_attr(feature = "cargo-clippy", allow(option_option))]
    media_type: Option<Option<Mime>>,
    _marker: PhantomData<(UnsafeCell<()>, Pinned)>,
}

impl Input {
    /// Create an instance of `Input` from components.
    ///
    /// Some fields remain uninitialized and their values are set when the corresponding
    /// method will be called.
    pub fn new(request: Request<ReqBody>) -> Input {
        Input {
            request,
            media_type: None,
            _marker: PhantomData,
        }
    }

    /// Return a shared reference to the value of raw HTTP request without the message body.
    pub fn request(&self) -> &Request<ReqBody> {
        &self.request
    }

    /// Takes the instance of `RequestBody` from this value.
    #[inline]
    pub fn payload(self: PinMut<'_, Self>) -> Option<Payload> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        this.request.body_mut().payload()
    }

    /// Attempts to get the entry of `Content-type` and parse its value.
    ///
    /// The result of this method is cached and it will return the reference to the cached value
    /// on subsequent calls.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_lifetimes))]
    pub fn content_type<'a>(
        self: PinMut<'a, Self>,
    ) -> Result<Option<&'a Mime>, InvalidContentType> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };

        match this.media_type {
            Some(ref m) => Ok(m.as_ref()),
            None => {
                let mime = match this.request.headers().get(http::header::CONTENT_TYPE) {
                    Some(raw) => {
                        let raw_str = raw
                            .to_str()
                            .map_err(|cause| InvalidContentType::DecodeToStr { cause })?;
                        let mime = raw_str
                            .parse()
                            .map_err(|cause| InvalidContentType::ParseToMime { cause })?;
                        Some(mime)
                    }
                    None => None,
                };
                Ok(this.media_type.get_or_insert(mime).as_ref())
            }
        }
    }
}

impl Deref for Input {
    type Target = Request<ReqBody>;

    fn deref(&self) -> &Self::Target {
        self.request()
    }
}

/// An error type during parsing the value of `Content-type` header.
#[derive(Debug, Fail)]
pub enum InvalidContentType {
    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    DecodeToStr { cause: http::header::ToStrError },

    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    ParseToMime { cause: mime::FromStrError },
}

impl HttpError for InvalidContentType {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
