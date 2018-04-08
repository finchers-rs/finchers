//! `Endpoint` layer

extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate http;

pub mod apply;
pub mod ext;

mod callable;
mod context;
mod endpoint;
mod error;

mod all;
mod ok;
mod skip_all;

// re-exports
pub use callable::Callable;
pub use context::{Context, Segment, Segments};
pub use endpoint::{endpoint, Endpoint, IntoEndpoint};
pub use error::{Error, ErrorKind};
pub use ext::EndpointExt;

pub use all::{all, All};
pub use ok::{ok, Ok};
pub use skip_all::{skip_all, SkipAll};
