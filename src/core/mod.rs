//! Low level HTTP definitions from Hyper

mod body;
mod segments;
mod request;

pub use self::body::{Body, BodyStream, FromBody};
pub use self::request::RequestParts;
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};

#[allow(missing_docs)]
pub trait FromHeader: ::hyper::header::Header + Clone {}

impl<H: ::hyper::header::Header + Clone> FromHeader for H {}
