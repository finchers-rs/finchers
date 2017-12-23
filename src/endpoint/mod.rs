//! Definition of the trait `Endpoint` and its implementors

pub mod method;

pub(crate) mod apply_fn;
pub(crate) mod body;
pub(crate) mod context;
pub(crate) mod endpoint;
pub(crate) mod header;
pub(crate) mod path;
pub(crate) mod result;

pub(crate) mod and_then;
pub(crate) mod from_err;
pub(crate) mod inspect;
pub(crate) mod join;
pub(crate) mod join_all;
pub(crate) mod map;
pub(crate) mod map_err;
pub(crate) mod or;
pub(crate) mod or_else;
pub(crate) mod skip;
pub(crate) mod skip_all;
pub(crate) mod then;
pub(crate) mod with;

// re-exports
pub use self::apply_fn::{apply_fn, ApplyFn};
pub use self::body::body;
pub use self::context::{EndpointContext, Segments};
pub use self::endpoint::{Endpoint, IntoEndpoint};
pub use self::header::{header, header_opt, EmptyHeader};
#[doc(inline)]
pub use self::method::MatchMethod;
pub use self::path::{path, paths};
pub use self::result::{err, ok, result, EndpointErr, EndpointOk, EndpointResult};

pub use self::and_then::AndThen;
pub use self::from_err::FromErr;
pub use self::inspect::Inspect;
pub use self::join::{Join, Join3, Join4, Join5, Join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::map::Map;
pub use self::map_err::MapErr;
pub use self::or::Or;
pub use self::or_else::OrElse;
pub use self::skip::Skip;
pub use self::skip_all::{skip_all, SkipAll};
pub use self::then::Then;
pub use self::with::With;
