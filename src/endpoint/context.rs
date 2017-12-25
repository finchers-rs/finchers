use std::path::{Component, Components, Path};
use tokio_core::reactor::Handle;
use request::Request;


#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Segments<'a>(Components<'a>);

impl<'a> From<&'a str> for Segments<'a> {
    fn from(path: &'a str) -> Self {
        let mut components = Path::new(path).components();
        components.next(); // skip the root ("/")
        Segments(components)
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| match c {
            Component::Normal(s) => s.to_str().unwrap(),
            _ => panic!("relatative path is not supported"),
        })
    }
}


/// A set of values, contains the incoming HTTP request and the finchers-specific context.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    handle: &'a Handle,
    segments: Option<Segments<'a>>,
}

impl<'a> EndpointContext<'a> {
    pub(crate) fn new(request: &'a Request, handle: &'a Handle) -> Self {
        EndpointContext {
            request,
            handle,
            segments: Some(Segments::from(request.path())),
        }
    }

    /// Returns the reference of HTTP request
    pub fn request(&self) -> &Request {
        self.request
    }

    /// Returns the reference of handle of the event loop in the running worker thread
    pub fn handle(&self) -> &'a Handle {
        self.handle
    }

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        self.segments.as_mut().and_then(|r| r.next())
    }

    /// Collect and return the remaining path segments, if available
    pub fn take_segments(&mut self) -> Option<Segments<'a>> {
        self.segments.take()
    }
}
