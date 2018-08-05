use error::HttpError;
use failure::Fail;
use http::{self, header, Request, StatusCode};
use mime::{self, Mime};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

use super::RequestBody;

/// The context which holds the received HTTP request.
///
/// The value is used throughout the processing in `Endpoint` and `Task`.
#[derive(Debug)]
pub struct Input {
    request: Request<RequestBody>,
    media_type: UnsafeCell<Option<Mime>>,
}

impl Input {
    /// Create an instance of `Input` from components.
    ///
    /// Some fields remain uninitialized and their values are set when the corresponding
    /// method will be called.
    pub fn new(request: Request<impl Into<RequestBody>>) -> Input {
        Input {
            request: request.map(Into::into),
            media_type: UnsafeCell::new(None),
        }
    }

    /// Return a shared reference to the value of raw HTTP request without the message body.
    pub fn request(&self) -> &Request<RequestBody> {
        &self.request
    }

    /// Return a mutable reference to the value of raw HTTP request without the message body.
    #[inline]
    pub fn request_mut(&mut self) -> &mut Request<RequestBody> {
        &mut self.request
    }

    /// Return the reference to the parsed media type in the request header.
    ///
    /// This method will perform parsing of the entry `Content-type` in the request header
    /// if it has not been done yet.  If the value is invalid, it will return an `Err`.
    pub fn media_type(&self) -> Result<Option<&Mime>, InvalidMediaType> {
        // safety: this mutable borrow is used only in the block.
        let media_type: &mut Option<Mime> = unsafe { &mut *self.media_type.get() };

        if media_type.is_none() {
            if let Some(raw) = self.request().headers().get(header::CONTENT_TYPE) {
                let raw_str = raw
                    .to_str()
                    .map_err(|cause| InvalidMediaType::DecodeToStr { cause })?;
                let mime = raw_str
                    .parse()
                    .map_err(|cause| InvalidMediaType::ParseToMime { cause })?;
                *media_type = Some(mime);
            }
        }

        Ok((&*media_type).as_ref())
    }
}

impl Deref for Input {
    type Target = Request<RequestBody>;

    fn deref(&self) -> &Self::Target {
        self.request()
    }
}

impl DerefMut for Input {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.request_mut()
    }
}

/// An error type which will be returned from `Input::media_type`.
#[derive(Debug, Fail)]
pub enum InvalidMediaType {
    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    DecodeToStr { cause: http::header::ToStrError },

    #[allow(missing_docs)]
    #[fail(display = "Content-type is invalid: {}", cause)]
    ParseToMime { cause: mime::FromStrError },
}

impl HttpError for InvalidMediaType {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn as_fail(&self) -> Option<&Fail> {
        Some(self)
    }
}
