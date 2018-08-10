//! Components for parsing the incoming HTTP request.

mod body;
mod global;
mod segments;
mod traits;

pub use self::body::{Data, PollDataError, RequestBody};
pub use self::global::with_get_cx;
pub use self::segments::{Cursor, EncodedStr, Segment};
pub use self::traits::{FromBody, FromHeader, FromQuery, FromSegment, QueryItems};

#[doc(hidden)]
pub use self::global::with_set_cx;

// ====

use std::mem::PinMut;
use std::ops::Deref;

use crate::error::HttpError;
use failure::Fail;
use http::{header, Request, StatusCode};
use mime::{self, Mime};

/// The context which holds the received HTTP request.
///
/// The value is used throughout the processing in `Endpoint` and `Task`.
#[derive(Debug)]
pub struct Input {
    request: Request<RequestBody>,
    // caches
    media_type: Option<Option<Mime>>,
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

    #[allow(missing_docs)]
    #[inline]
    pub fn take_body(self: PinMut<'a, Self>) -> RequestBody {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        this.request.body_mut().take()
    }

    /// Return the reference to the parsed media type in the request header.
    ///
    /// This method will perform parsing of the entry `Content-type` in the request header
    /// if it has not been done yet.  If the value is invalid, it will return an `Err`.
    pub fn media_type(self: PinMut<'a, Self>) -> Result<Option<&'a Mime>, InvalidMediaType> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };

        match this.media_type {
            Some(ref m) => Ok(m.as_ref()),
            None => {
                let mime = match this.request.headers().get(header::CONTENT_TYPE) {
                    Some(raw) => {
                        let raw_str = raw
                            .to_str()
                            .map_err(|cause| InvalidMediaType::DecodeToStr { cause })?;
                        let mime = raw_str
                            .parse()
                            .map_err(|cause| InvalidMediaType::ParseToMime { cause })?;
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

/// An error type which will be returned from `Input::media_type`.
#[derive(Debug, Fail)]
pub enum InvalidMediaType {
    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    DecodeToStr { cause: header::ToStrError },

    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    ParseToMime { cause: mime::FromStrError },
}

impl HttpError for InvalidMediaType {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
