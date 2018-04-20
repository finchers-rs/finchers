#![doc(html_url = "https://docs.rs/finchers-http/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(warnings)]

extern crate finchers_core;
extern crate http;
#[macro_use]
extern crate futures;
extern crate bytes;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;

pub mod body;
pub mod header;
pub mod json;
pub mod method;
pub mod path;
pub mod query;

pub use body::FromData;
pub use header::FromHeader;
pub use path::{FromSegment, FromSegments};
