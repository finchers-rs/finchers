//! Components for parsing the incoming HTTP request.

mod body;
mod global;
mod header;
#[doc(hidden)]
pub mod local_map;
mod segments;
mod traits;

pub use self::body::{Data, PollDataError, RequestBody};
pub use self::global::with_get_cx;
pub use self::header::FromHeaderValue;
pub use self::segments::{Cursor, EncodedStr, Segment};
pub use self::traits::{FromBody, FromQuery, FromSegment, QueryItems};

#[doc(hidden)]
pub use self::global::with_set_cx;

// ====

use std::cell::UnsafeCell;
use std::marker::{PhantomData, Pinned};
use std::mem::PinMut;
use std::ops::Deref;

use failure::Fail;
use http::{Request, StatusCode};
use mime::{self, Mime};

use self::local_map::LocalMap;
use crate::error::HttpError;

/// The contextual information with an incoming HTTP request.
#[derive(Debug)]
pub struct Input {
    request: Request<RequestBody>,
    // caches
    media_type: Option<Option<Mime>>,
    locals: LocalMap,
    _marker: PhantomData<(UnsafeCell<()>, Pinned)>,
}

impl Input {
    /// Create an instance of `Input` from components.
    ///
    /// Some fields remain uninitialized and their values are set when the corresponding
    /// method will be called.
    pub fn new(request: Request<impl Into<RequestBody>>) -> Input {
        Input {
            request: request.map(Into::into),
            media_type: None,
            locals: LocalMap::default(),
            _marker: PhantomData,
        }
    }

    /// Return a shared reference to the value of raw HTTP request without the message body.
    pub fn request(&self) -> &Request<RequestBody> {
        &self.request
    }

    /// Return a mutable reference to the value of raw HTTP request without the message body.
    #[inline]
    pub fn request_pinned_mut(self: PinMut<'a, Self>) -> PinMut<'a, Request<RequestBody>> {
        unsafe { PinMut::map_unchecked(self, |input| &mut input.request) }
    }

    /// Takes the instance of `RequestBody` from this value.
    #[inline]
    pub fn body(self: PinMut<'a, Self>) -> RequestBody {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        this.request.body_mut().take()
    }

    /// Attempts to get the entry of `Content-type` and parse its value.
    ///
    /// The result of this method is cached and it will return the reference to the cached value
    /// on subsequent calls.
    pub fn content_type(self: PinMut<'a, Self>) -> Result<Option<&'a Mime>, InvalidContentType> {
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
    type Target = Request<RequestBody>;

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
