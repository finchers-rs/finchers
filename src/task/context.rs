use tokio_core::reactor::Handle;
use http::{Body, Cookies, Request};

/// The context during `Task::launch`.
#[derive(Debug)]
pub struct TaskContext<'a> {
    pub(crate) request: &'a mut Request,
    pub(crate) handle: &'a Handle,
    pub(crate) cookies: &'a mut Cookies,
    pub(crate) body: Option<Body>,
}

impl<'a> TaskContext<'a> {
    /// Returns the reference of `Request`.
    pub fn request(&mut self) -> &mut Request {
        &mut *self.request
    }

    /// Takes the instance of `Body` from this context, if available.
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }

    /// Returns the reference of the handle of event loop.
    pub fn handle(&self) -> &Handle {
        &*self.handle
    }

    /// Returns the reference of Cookie jar.
    pub fn cookies(&mut self) -> &mut Cookies {
        &mut *self.cookies
    }
}
