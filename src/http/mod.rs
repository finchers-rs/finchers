//! Low level HTTP definitions from Hyper

mod from_body;
mod from_header;
mod into_response;
mod segments;

pub use hyper::{Body, Chunk, Error};
pub use hyper::{Response, StatusCode};
pub use hyper::header::Headers;
pub use http_crate::Response as HttpResponse;

pub use self::from_body::FromBody;
pub use self::from_header::FromHeader;
pub use self::into_response::IntoResponse;
pub use self::segments::{FromSegment, FromSegments, Segment, Segments};

#[allow(missing_docs)]
pub type Request = ::http_crate::Request<Option<Body>>;
