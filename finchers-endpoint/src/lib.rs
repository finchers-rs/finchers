//! `Endpoint` layer

extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate http;

pub mod body;
pub mod header;
pub mod method;
pub mod path;

mod context;
mod endpoint;

mod and_then;
mod join;
mod join_all;
mod map;
mod ok;
mod or;
mod skip;
mod skip_all;
mod with;

// re-exports
pub use context::Context;
pub use endpoint::{endpoint, Endpoint, EndpointFuture, IntoEndpoint};

pub use and_then::AndThen;
pub use join::{Join, Join3, Join4, Join5};
pub use join_all::{join_all, JoinAll};
pub use map::Map;
pub use ok::{ok, EndpointOk};
pub use or::Or;
pub use skip::Skip;
pub use skip_all::{skip_all, SkipAll};
pub use with::With;
