//! Primitive HTTP types and traits

mod from_body;
mod from_header;
mod into_response;
mod segments;

pub use hyper::{Body, Chunk, Error};

pub use self::from_body::FromBody;
pub use self::from_header::FromHeader;
pub use self::into_response::IntoResponse;
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};
