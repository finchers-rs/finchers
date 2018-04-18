use super::{Error, ErrorKind, RequestBody};
use http::{header, Request};
use mime::Mime;
use std::cell::UnsafeCell;

scoped_thread_local!(static CURRENT_INPUT: Input);

/// The context which holds the received HTTP request.
///
/// The value is used throughout the processing like "Endpoint" and "Task".
#[derive(Debug)]
pub struct Input {
    request: Request<()>,
    media_type: UnsafeCell<Option<Mime>>,

    body: Option<RequestBody>,
}

impl Input {
    /// Create an instance of "Input" from components.
    ///
    /// Some fields remain uninitialized and their values are set when the corresponding
    /// method will be called.
    pub fn new(request: Request<()>, body: RequestBody) -> Input {
        Input {
            request,
            media_type: UnsafeCell::new(None),

            body: Some(body),
        }
    }

    /// Set the reference to itself to the thread-local storage and execute given closure.
    ///
    /// Typically, this method is used in the implementation of "Task" which holds some closures.
    pub fn enter_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CURRENT_INPUT.set(self, f)
    }

    /// Execute a closure with the reference to the instance of "Input" from the thread-local storage.
    ///
    /// This method is only used in a closure passed to "enter_scope".
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

    /// Take away the instance of "RequestBody" from this context.
    ///
    /// If the instance has been already removed, the method will return a "None".
    pub fn body(&mut self) -> Option<RequestBody> {
        self.body.take()
    }

    /// Return the reference to the parsed media type in the request header.
    ///
    /// This method will perform parsing of the entry "Content-type" in the request header
    /// if it has not been done yet.  If the value is invalid, it will return an "Err".
    pub fn media_type(&self) -> Result<Option<&Mime>, Error> {
        // safety: this mutable borrow is used only in the block.
        let media_type: &mut Option<Mime> = unsafe { &mut *self.media_type.get() };

        if media_type.is_none() {
            if let Some(raw) = self.request().headers().get(header::CONTENT_TYPE) {
                let raw_str = raw.to_str().map_err(ErrorKind::DecodeHeaderToStr)?;
                let mime = raw_str.parse().map_err(ErrorKind::ParseMediaType)?;

                *media_type = Some(mime);
            }
        }

        Ok((&*media_type).as_ref())
    }
}
