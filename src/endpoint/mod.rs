//! Definition of the trait `Endpoint` and its implementors

pub mod method;

mod apply_fn;
mod body;
mod context;
mod endpoint;
mod from_param;
mod header;
mod lazy;
mod path;
mod query;
mod reject;
mod result;

mod and_then;
mod from_err;
mod inspect;
mod join;
mod join_all;
mod map;
mod map_err;
mod or;
mod or_else;
mod skip;
mod skip_all;
mod then;
mod with;

// re-exports
pub use self::apply_fn::{apply_fn, ApplyFn};
pub use self::body::body;
pub use self::from_param::FromParam;
pub use self::context::EndpointContext;
pub use self::endpoint::{Endpoint, EndpointError, IntoEndpoint};
pub use self::header::{header, header_opt};
pub use self::lazy::{lazy, Lazy};
pub use self::method::MatchMethod;
pub use self::query::{query, query_opt};
pub use self::path::{param, params, segment};
pub use self::reject::{reject, Reject};
pub use self::result::{err, ok, result, EndpointErr, EndpointOk, EndpointResult};

pub use self::and_then::{and_then, AndThen};
pub use self::from_err::{from_err, FromErr};
pub use self::inspect::{inspect, Inspect};
pub use self::join::{join, Join, Join3, Join4, Join5, Join6, join3, join4, join5, join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::map::{map, Map};
pub use self::map_err::{map_err, MapErr};
pub use self::or::{or, Or};
pub use self::or_else::{or_else, OrElse};
pub use self::skip::{skip, Skip};
pub use self::skip_all::{skip_all, SkipAll};
pub use self::then::{then, Then};
pub use self::with::{with, With};
