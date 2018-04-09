//! `Endpoint` layer

extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate http;

pub mod apply;

mod abort;
mod abort_with;
mod all;
mod and;
mod callable;
mod chain;
mod ext;
mod context;
mod endpoint;
mod error;
mod left;
mod map;
mod ok;
mod or;
mod right;
mod skip_all;
mod then;
mod try_abort;

// re-exports
pub use abort::Abort;
pub use abort_with::AbortWith;
pub use all::{all, All};
pub use and::And;
pub use callable::Callable;
pub use context::{Context, Segment, Segments};
pub use endpoint::{endpoint, Endpoint, IntoEndpoint};
pub use error::{Error, ErrorKind};
pub use ext::EndpointExt;
pub use left::Left;
pub use map::Map;
pub use ok::{ok, Ok};
pub use or::Or;
pub use right::Right;
pub use skip_all::{skip_all, SkipAll};
pub use then::Then;
pub use try_abort::TryAbort;
