use super::{BodyStream, Error, ErrorKind};
use http::request::Parts;
use http::{Extensions, Request};
use http::{header, HeaderMap, Method, Uri, Version};
use mime::Mime;
use std::cell::RefCell;
use std::mem;

task_local!(static CURRENT_INPUT: RefCell<Option<Input>> = RefCell::new(None));

/// Insert the value of `Input` to the current task context.
///
/// This function will return a `Some(input)` if the value of `Input` has already set.
pub fn replace_input(input: Option<Input>) -> Option<Input> {
    CURRENT_INPUT.with(|i| mem::replace(&mut *i.borrow_mut(), input))
}

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Input {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    extensions: Extensions,
    body: Option<BodyStream>,
    media_type: Option<Mime>,
}

impl<B> From<Request<B>> for Input
where
    B: Into<BodyStream>,
{
    fn from(request: Request<B>) -> Self {
        let request = request.map(Into::into);
        let (
            Parts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = request.into_parts();
        Input {
            method,
            uri,
            version,
            headers,
            extensions,
            body: Some(body),
            media_type: None,
        }
    }
}

impl Input {
    /// Run a closure with the reference to `Input` at the current task context.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Input) -> R,
    {
        CURRENT_INPUT.with(|input| {
            let input = input.borrow();
            let input = input.as_ref().expect("The instance of Input has not initialized yet.");
            f(input)
        })
    }

    /// Run a closure with the mutable reference to `Input` at the current task context.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Input) -> R,
    {
        CURRENT_INPUT.with(|input| {
            let mut input = input.borrow_mut();
            let input = input.as_mut().expect("The instance of Input has not initialized yet.");
            f(input)
        })
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn path(&self) -> &str {
        self.uri().path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri().query()
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn body(&mut self) -> Option<BodyStream> {
        self.body.take()
    }

    pub fn media_type(&mut self) -> Result<Option<&Mime>, Error> {
        if self.media_type.is_none() && self.headers().contains_key(header::CONTENT_TYPE) {
            let mime = {
                let raw = self.headers().get(header::CONTENT_TYPE).unwrap();
                raw.to_str()
                    .map_err(ErrorKind::DecodeHeaderToStr)?
                    .parse()
                    .map_err(ErrorKind::ParseMediaType)?
            };
            self.media_type = Some(mime);
        }
        Ok(self.media_type.as_ref())
    }
}
