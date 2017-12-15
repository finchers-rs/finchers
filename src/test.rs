//! Helper functions for testing

use futures::{Future, Poll};
use hyper::Method;
use hyper::header::Header;
use tokio_core::reactor::Core;

use context::{Context, RequestInfo};
use endpoint::{Endpoint, EndpointError};
use request::{Body, Request};
use task::Task;


/// A test case for `run_test()`
#[derive(Debug)]
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

impl Into<RequestInfo> for TestCase {
    fn into(self) -> RequestInfo {
        let req = self.request;
        let body = self.body.unwrap_or_default();
        RequestInfo::new(req, body)
    }
}


/// Invoke given endpoint and return its result
pub fn run_test<T, E>(endpoint: T, input: TestCase) -> Result<Result<E::Item, E::Error>, EndpointError>
where
    T: AsRef<E>,
    E: Endpoint,
{
    let mut core = Core::new().unwrap();

    let mut ctx = Context::new(input.into());
    let task = endpoint.as_ref().apply(&mut ctx)?;

    Ok(core.run(TestFuture { task, ctx }))
}

#[derive(Debug)]
struct TestFuture<T: Task> {
    task: T,
    ctx: Context,
}

impl<T: Task> Future for TestFuture<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.task.poll(&mut self.ctx)
    }
}
