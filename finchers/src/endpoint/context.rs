use request::{Input, Segments};

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    segments: Segments<'a>,
}

impl<'a> EndpointContext<'a> {
    pub(crate) fn new(input: &'a Input) -> Self {
        EndpointContext {
            segments: Segments::from(input.path()),
        }
    }

    /// Returns the reference of remaining path segments
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}
