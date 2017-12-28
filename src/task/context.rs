use tokio_core::reactor::Handle;
use http::{Body, Cookies, Request};

#[derive(Debug)]
pub struct TaskContext<'a> {
    pub(crate) request: &'a Request,
    pub(crate) handle: &'a Handle,
    pub(crate) cookies: &'a mut Cookies,
    pub(crate) body: Option<Body>,
}

impl<'a> TaskContext<'a> {
    pub fn request(&self) -> &'a Request {
        self.request
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }

    pub fn handle(&self) -> &'a Handle {
        self.handle
    }

    pub fn cookies(&mut self) -> &mut Cookies {
        &mut *self.cookies
    }
}
