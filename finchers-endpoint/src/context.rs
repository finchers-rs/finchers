use finchers_core::Input;
use path::Segments;

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    segments: Segments<'a>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(input: &'a Input) -> Self {
        Context {
            segments: Segments::from(input.path()),
        }
    }

    /// Returns the reference of remaining path segments
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}
