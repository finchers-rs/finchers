use tokio_core::reactor::Handle;
use http::{Body, Request};


#[derive(Debug)]
pub struct TaskContext<'a> {
    request: &'a Request,
    handle: &'a Handle,
    body: Option<Body>,
}

impl<'a> TaskContext<'a> {
    pub(crate) fn new(request: &'a Request, handle: &'a Handle, body: Body) -> Self {
        Self {
            request,
            handle,
            body: Some(body),
        }
    }

    pub fn request(&self) -> &Request {
        &self.request
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }

    pub fn handle(&self) -> &'a Handle {
        self.handle
    }
}
