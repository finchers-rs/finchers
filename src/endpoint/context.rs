use std::ops::Deref;
use http::{Request, Segments};

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    segments: Segments<'a>,
}

impl<'a> EndpointContext<'a> {
    pub(crate) fn new(request: &'a Request) -> Self {
        EndpointContext {
            request,
            segments: Segments::from(request.path()),
        }
    }

    /// Returns the reference of HTTP request
    pub fn request(&self) -> &Request {
        self.request
    }

    /// Returns the reference of remaining path segments
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}

impl<'a> Deref for EndpointContext<'a> {
    type Target = Request;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.request
    }
}
