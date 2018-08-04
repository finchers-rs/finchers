use error::HttpError;
use failure::Fail;
use http::{self, header, Request, StatusCode};
use mime::{self, Mime};
use std::cell::UnsafeCell;

scoped_thread_local!(static CURRENT_INPUT: Input);

/// The context which holds the received HTTP request.
///
/// The value is used throughout the processing in `Endpoint` and `Task`.
#[derive(Debug)]
pub struct Input {
    request: Request<()>,
    media_type: UnsafeCell<Option<Mime>>,
}

impl Input {
    /// Create an instance of `Input` from components.
    ///
    /// Some fields remain uninitialized and their values are set when the corresponding
    /// method will be called.
    pub fn new(request: Request<()>) -> Input {
        Input {
            request,
            media_type: UnsafeCell::new(None),
        }
    }

    /// Set the reference to itself to the thread-local storage and execute given closure.
    ///
    /// Typically, this method is used in the implementation of `Task` which holds some closures.
    pub fn enter_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CURRENT_INPUT.set(self, f)
    }

    /// Execute a closure with the reference to the instance of `Input` from the thread-local storage.
    ///
    /// This method is only used in a closure passed to `enter_scope`.
    /// Otherwise, it will be panic.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Input) -> R,
    {
        CURRENT_INPUT.with(|input| f(input))
    }

    /// Return a shared reference to the value of raw HTTP request without the message body.
    pub fn request(&self) -> &Request<()> {
        &self.request
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
