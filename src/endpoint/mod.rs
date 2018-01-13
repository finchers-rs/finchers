//! `Endpoint` layer

pub mod method;

pub(crate) mod body;
pub(crate) mod chain;
pub(crate) mod context;
pub(crate) mod endpoint;
pub(crate) mod header;
pub(crate) mod path;
pub(crate) mod stream;

pub(crate) mod and_then;
pub(crate) mod err;
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
pub use self::body::{body, Body};
pub use self::context::{EndpointContext, Segment, Segments};
pub use self::endpoint::{Endpoint, EndpointResult, IntoEndpoint};
pub use self::header::{header, header_opt};
#[doc(inline)]
pub use self::method::MatchMethod;
pub use self::path::{match_, path, paths, ExtractPath, ExtractPaths, MatchPath};
pub use self::stream::{body_stream, BodyStream};

pub use self::and_then::AndThen;
pub use self::err::{err, EndpointErr};
pub use self::join::{Join, Join3, Join4, Join5};
pub use self::join_all::{join_all, JoinAll};
pub use self::map::Map;
pub use self::map_err::MapErr;
pub use self::ok::{ok, EndpointOk};
pub use self::or::Or;
pub use self::skip::Skip;
pub use self::skip_all::{skip_all, SkipAll};
pub use self::with::With;
