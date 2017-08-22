//! Helper functions for testing

use std::cell::RefCell;
use hyper::Method;
use hyper::header::Header;
use tokio_core::reactor::Core;

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::{Body, Request};

/// A test case for `run_test()`
pub struct TestCase {
    request: Request,
    body: Option<Body>,
}

impl TestCase {
    /// Construct a `TestCase` from given HTTP method and URI
    pub fn new(method: Method, uri: &str) -> Self {
        let request = Request::new(method, uri).expect("invalid URI");
        Self {
            request,
            body: None,
        }
    }

    /// Equivalent to `TestCase::new(Method::Get, uri)`
    pub fn get(uri: &str) -> Self {
        Self::new(Method::Get, uri)
    }

    /// Equivalent to `TestCase::new(Method::Post, uri)`
    pub fn post(uri: &str) -> Self {
        Self::new(Method::Post, uri)
    }

    /// Equivalent to `TestCase::new(Method::Put, uri)`
    pub fn put(uri: &str) -> Self {
        Self::new(Method::Put, uri)
    }

    /// Equivalent to `TestCase::new(Method::Delete, uri)`
    pub fn delete(uri: &str) -> Self {
        Self::new(Method::Delete, uri)
    }

    /// Equivalent to `TestCase::new(Method::Patch, uri)`
    pub fn patch(uri: &str) -> Self {
        Self::new(Method::Patch, uri)
    }

    /// Set the HTTP header of this test case
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.request.headers.set(header);
        self
    }

    /// Set the request body of this test case
    pub fn with_body<B: Into<Body>>(mut self, body: B) -> Self {
        self.body = Some(body.into());
        self
    }
}


/// Invoke given endpoint and return its result
pub fn run_test<E: Endpoint>(endpoint: E, input: TestCase) -> Result<E::Item, FinchersError> {
    let req = input.request;
    let body = RefCell::new(Some(input.body.unwrap_or_default()));
    let ctx = Context::new(&req, &body);

    let (_ctx, f) = endpoint.apply(ctx);
    let f = f?;

    let mut core = Core::new().unwrap();
    core.run(f)
}
