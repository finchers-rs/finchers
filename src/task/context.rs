use http::{Body, Request};

/// The context during `Task::launch`.
#[derive(Debug)]
pub struct TaskContext<'a> {
    pub(crate) request: &'a mut Request,
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
}
