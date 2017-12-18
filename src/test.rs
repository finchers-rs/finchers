//! Utilities for testing

use futures::{Future, Poll};
use hyper::Method;
use hyper::header::Header;
use tokio_core::reactor::Core;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use request::{Body, Request};
use task::{Task, TaskContext};


#[allow(missing_docs)]
#[derive(Debug)]
pub struct TestCase {
    request: Request,
    body: Option<Body>,
}

#[allow(missing_docs)]
impl TestCase {
    pub fn new(method: Method, uri: &str) -> Self {
        let request = Request::new(method, uri).expect("invalid URI");
        Self {
            request,
            body: None,
        }
    }

    pub fn get(uri: &str) -> Self {
        Self::new(Method::Get, uri)
    }

    pub fn post(uri: &str) -> Self {
        Self::new(Method::Post, uri)
    }

    pub fn put(uri: &str) -> Self {
        Self::new(Method::Put, uri)
    }

    pub fn delete(uri: &str) -> Self {
        Self::new(Method::Delete, uri)
    }

    pub fn patch(uri: &str) -> Self {
        Self::new(Method::Patch, uri)
    }

    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.request.headers.set(header);
        self
    }

    pub fn with_body<B: Into<Body>>(mut self, body: B) -> Self {
        self.body = Some(body.into());
        self
    }
}


/// Run the endpoint with a test case.
///
/// # Example
///
/// ```ignore
/// let endpoint = ...;
///
/// let input = TestCase::get("/foo/bar")
///     .with_body(json!({ ... }).to_string());
///
/// assert_eq!(run_test(&endpoint, input), Ok(..));
/// ```
pub fn run_test<E: Endpoint>(endpoint: &E, input: TestCase) -> Result<E::Item, E::Error>
where
    E::Error: From<EndpointError>,
{
    let mut core = Core::new().unwrap();

    let TestCase { request, body } = input;

    let task = {
        let mut ctx = EndpointContext::new(&request);
        endpoint.apply(&mut ctx)?
    };
    let ctx = TaskContext::new(request, body.unwrap_or_default());
    core.run(TestFuture { task, ctx })
}

#[derive(Debug)]
struct TestFuture<T: Task> {
    task: T,
    ctx: TaskContext,
}

impl<T: Task> Future for TestFuture<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.task.poll(&mut self.ctx)
    }
}
