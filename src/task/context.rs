use request::{Body, Request};


#[derive(Debug)]
pub struct TaskContext {
    request: Request,
    body: Option<Body>,
}

impl TaskContext {
    pub fn new(request: Request, body: Body) -> Self {
        Self {
            request,
            body: Some(body),
        }
    }

    pub fn request(&self) -> &Request {
        &self.request
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}
