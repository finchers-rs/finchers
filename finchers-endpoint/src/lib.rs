//! `Endpoint` layer

extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate http;

pub mod apply;
pub mod body;
pub mod header;
pub mod method;
pub mod path;
pub mod ext;

mod context;
mod endpoint;

mod ok;
mod join_all;
mod skip_all;

// re-exports
pub use context::Context;
pub use endpoint::{endpoint, Endpoint, IntoEndpoint};
pub use ext::EndpointExt;

pub use ok::{ok, EndpointOk};
pub use join_all::{join_all, JoinAll};
pub use skip_all::{skip_all, SkipAll};

pub use body::FromBody;
pub use header::FromHeader;
pub use path::{FromSegment, FromSegments};
