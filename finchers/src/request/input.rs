use std::cell::RefCell;
use std::ops::Deref;

use futures::task::LocalKey;
use http::request::Parts;
use http::{Extensions, Request};
use http::{header, HeaderMap, Method, Uri, Version};
use mime;

use request::body::{Body, BodyStream};

task_local!(static INPUT: RefCell<Option<Input>> = RefCell::new(None));

pub fn input_key() -> &'static LocalKey<RefCell<Option<Input>>> {
    &INPUT
}

#[allow(missing_docs)]
pub fn with_input<F, R>(f: F) -> R
where
    F: FnOnce(&Input) -> R,
{
    INPUT.with(|input| {
        let input = input.borrow();
        let input = input
            .as_ref()
            .expect("The instance of Input has not initialized yet.");
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
        let input = input
            .as_mut()
            .expect("The instance of Input has not initialized yet.");
        f(input)
    })
}

/// The value of incoming HTTP request
#[derive(Debug)]
pub struct Input {
    parts: RequestParts,
    body: Option<BodyStream>,
    extensions: Extensions,
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
            parts: RequestParts {
                method,
                uri,
                version,
                headers,
            },
            body: Some(body),
            extensions: extensions,
        }
    }
}

impl Input {
    #[allow(missing_docs)]
    pub fn parts(&self) -> &RequestParts {
        &self.parts
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

    #[allow(missing_docs)]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[allow(missing_docs)]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

impl Deref for Input {
    type Target = RequestParts;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.parts()
    }
}

/// Clonable, shared parts in the incoming HTTP request
#[derive(Debug)]
pub struct RequestParts {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
}

#[allow(missing_docs)]
impl RequestParts {
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

    pub fn media_type(&self) -> Option<mime::Mime> {
        self.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|s| s.to_str().ok().and_then(|s| s.parse().ok()))
    }
}
