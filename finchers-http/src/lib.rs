#![doc(html_root_url = "https://docs.rs/finchers-http/0.11.0")]

extern crate bytes;
#[macro_use]
extern crate finchers_core;
extern crate http;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate percent_encoding;

pub mod body;
pub mod header;
pub mod json;
pub mod method;
pub mod path;
pub mod query;

pub use body::FromBody;
pub use header::FromHeader;
pub use path::{FromSegment, FromSegments};
pub use query::FromQuery;
