//! Low level HTTP definitions from Hyper

mod header;
mod segments;
mod request;

pub use self::header::FromHeader;
pub use self::request::RequestParts;
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};
