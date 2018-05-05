//! Components for constructing `Endpoint`.

mod apply;
mod context;
mod endpoint;

// re-exports
pub use self::apply::ApplyRequest;
pub use self::context::{Context, Segment, Segments};
pub use self::endpoint::{Endpoint, IntoEndpoint};

/// An identity function for giving additional trait bound to an endpoint.alloc
///
/// This function is usually used in the implementation of library.
#[inline(always)]
pub fn assert_output<E, T>(endpoint: E) -> E
where
    E: Endpoint<Output = T>,
{
    endpoint
}
