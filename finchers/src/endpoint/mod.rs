//! `Endpoint` layer

pub mod body;
pub mod header;
pub mod method;
pub mod path;

mod context;
mod endpoint;
mod outcome;

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
pub use self::context::EndpointContext;
pub use self::endpoint::{endpoint, Endpoint, EndpointFuture, IntoEndpoint};
pub use self::outcome::Outcome;

pub use self::and_then::AndThen;
pub use self::join::{Join, Join3, Join4, Join5};
pub use self::join_all::{join_all, JoinAll};
pub use self::map::Map;
pub use self::ok::{ok, EndpointOk};
pub use self::or::Or;
pub use self::skip::Skip;
pub use self::skip_all::{skip_all, SkipAll};
pub use self::with::With;

/// The "prelude" for building endpoints
pub mod prelude {
    pub use super::body::{body, body_stream};
    pub use super::endpoint::{endpoint, Endpoint, IntoEndpoint};
    pub use super::header::{header, header_opt, header_req};
    pub use super::method::{delete, get, head, patch, post, put};
    pub use super::path::{match_, path, paths};
}
