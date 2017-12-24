#![allow(missing_docs)]

mod into_body;
mod responder;

pub use hyper::header;
pub use hyper::mime;
pub use hyper::StatusCode;

pub use self::into_body::IntoBody;
pub use self::responder::{respond, Responder};
