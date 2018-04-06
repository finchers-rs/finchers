pub mod body;
pub use self::body::{Body, BodyStream};

use http::request::Parts;
use http::{Extensions, Request};
use http::{header, HeaderMap, Method, Uri, Version};
use mime;
use std::cell::RefCell;

task_local!(static INPUT: RefCell<Option<Input>> = RefCell::new(None));

#[allow(missing_docs)]
pub fn set_input(input: Input) {
    INPUT.with(|i| {
        i.borrow_mut().get_or_insert(input);
    })
}

#[allow(missing_docs)]
pub fn with_input<F, R>(f: F) -> R
where
    F: FnOnce(&Input) -> R,
{
    INPUT.with(|input| {
        let input = input.borrow();
        let input = input.as_ref().expect("The instance of Input has not initialized yet.");
        f(input)
    })
}

#[allow(missing_docs)]
pub fn with_input_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Input) -> R,
{
    INPUT.with(|input| {
        let mut input = input.borrow_mut();
        let input = input.as_mut().expect("The instance of Input has not initialized yet.");
        f(input)
    })
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
        }
    }
}

impl Input {
    #[allow(missing_docs)]
    pub fn method(&self) -> &Method {
        &self.method
    }

    #[allow(missing_docs)]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    #[allow(missing_docs)]
    pub fn path(&self) -> &str {
        self.uri().path()
    }

    #[allow(missing_docs)]
    pub fn query(&self) -> Option<&str> {
        self.uri().query()
    }

    #[allow(missing_docs)]
    pub fn version(&self) -> &Version {
        &self.version
    }

    #[allow(missing_docs)]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[allow(missing_docs)]
    pub fn media_type(&self) -> Option<mime::Mime> {
        self.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|s| s.to_str().ok().and_then(|s| s.parse().ok()))
    }

    #[allow(missing_docs)]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[allow(missing_docs)]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    #[allow(missing_docs)]
    pub fn body(&mut self) -> Option<Body> {
        if let Some(stream) = self.body.take() {
            self.extensions.insert(Body::from(stream));
        }
        self.extensions.get::<Body>().cloned()
    }

    #[allow(missing_docs)]
    pub fn body_stream(&mut self) -> Option<BodyStream> {
        self.body.take().map(Into::into)
    }
}
