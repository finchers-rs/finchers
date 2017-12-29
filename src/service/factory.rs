#![allow(missing_docs)]

use std::io;
use tokio_core::reactor::Handle;
use tokio_service::{NewService, Service};
use hyper;

/// A factory of Hyper's `Service`.
pub trait ServiceFactory {
    type Service: Service<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error>;

    /// Creates an instance of Hyper's `Service` with given handle of event loop in the current thread.
    fn new_service(&self, handle: &Handle) -> io::Result<Self::Service>;
}

impl<S> ServiceFactory for S
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error>,
{
    type Service = S::Instance;

    fn new_service(&self, _: &Handle) -> io::Result<Self::Service> {
        NewService::new_service(self)
    }
}
