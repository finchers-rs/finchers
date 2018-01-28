//! `Endpoint` layer

pub mod body;
pub mod method;
pub mod header;
pub mod path;

pub(crate) mod and_then;
pub(crate) mod context;
pub(crate) mod chain;
pub(crate) mod endpoint;
pub(crate) mod err;
pub(crate) mod from_err;
pub(crate) mod from_ok;
pub(crate) mod from_ok_err;
pub(crate) mod join;
pub(crate) mod join_all;
pub(crate) mod map;
pub(crate) mod map_err;
pub(crate) mod or;
pub(crate) mod ok;
pub(crate) mod skip;
pub(crate) mod skip_all;
pub(crate) mod with;

// re-exports
pub use self::and_then::AndThen;
pub use self::context::EndpointContext;
pub use self::endpoint::{endpoint, Endpoint, EndpointError, EndpointResult, IntoEndpoint};
pub use self::err::{err, EndpointErr};
pub use self::from_err::FromErr;
pub use self::from_ok::FromOk;
pub use self::from_ok_err::FromOkErr;
pub use self::join::{Join, Join3, Join4, Join5};
pub use self::join_all::{join_all, JoinAll};
pub use self::map::Map;
pub use self::map_err::MapErr;
pub use self::ok::{ok, EndpointOk};
pub use self::or::Or;
pub use self::skip::Skip;
pub use self::skip_all::{skip_all, SkipAll};
pub use self::with::With;

/// The "prelude" for building endpoints
pub mod prelude {
    pub use super::endpoint::{endpoint, Endpoint, IntoEndpoint};
    pub use super::body::{body, body_stream};
    pub use super::header::{header, header_opt, header_req};
    pub use super::path::{match_, path, paths};
    pub use super::method::{delete, get, head, patch, post, put};
}
