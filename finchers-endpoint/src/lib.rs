//! `Endpoint` layer

extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate http;

pub mod apply;
pub mod body;
pub mod ext;
pub mod header;
pub mod method;
pub mod path;

mod callable;
mod context;
mod endpoint;
mod error;

mod all;
mod ok;
mod skip_all;

// re-exports
pub use callable::Callable;
pub use context::Context;
pub use endpoint::{endpoint, Endpoint, IntoEndpoint};
pub use error::{Error, ErrorKind};
pub use ext::EndpointExt;

pub use all::{all, All};
pub use ok::{ok, Ok};
pub use skip_all::{skip_all, SkipAll};

pub use body::FromBody;
pub use header::FromHeader;
pub use path::{FromSegment, FromSegments};
