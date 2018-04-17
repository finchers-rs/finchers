use http::StatusCode;

mod body;
mod responder;

pub use self::body::Body;
pub use self::responder::{Debug, Output, Responder};

/// Trait which represents an HTTP status associated with the types.
pub trait HttpStatus {
    /// Returns a HTTP status code associated with this type
    fn status_code(&self) -> StatusCode;
}
