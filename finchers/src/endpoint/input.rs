use std::cell::RefCell;
use std::ops::Deref;

use futures::task::LocalKey;
use http::request::Parts;
use http::{Extensions, Request};

use request::RequestParts;
use request::body::{Body, BodyStream};

task_local!(static INPUT: RefCell<Option<Input>> = RefCell::new(None));

pub fn input_key() -> &'static LocalKey<RefCell<Option<Input>>> {
    &INPUT
}

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
            parts: RequestParts::new(method, uri, version, headers),
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

    #[allow(missing_docs)]
    pub fn shared_parts(&mut self) -> (RequestParts, Body) {
        let parts = self.parts.clone();
        let body = self.body().expect("cannot take the request body twice");
        (parts, body)
    }
}

impl Deref for Input {
    type Target = RequestParts;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.parts()
    }
}
