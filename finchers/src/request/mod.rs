//! Low level HTTP definitions from Hyper

pub mod body;
mod header;
mod input;
mod segments;

pub use self::body::FromBody;
pub use self::header::FromHeader;
pub use self::input::{with_input, with_input_mut, Input, RequestParts};
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};

pub(crate) use self::input::set_input;
