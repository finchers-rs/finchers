//! Definition of the trait `Endpoint` and its implementors

mod endpoint;

pub mod body;
pub mod combinator;
pub mod header;
pub mod method;
pub mod path;
pub mod query;

// re-exports
#[doc(inline)]
pub use self::endpoint::{Endpoint, EndpointError};

#[doc(inline)]
pub use self::body::body;

#[doc(inline)]
pub use self::header::{header, header_opt};

#[doc(inline)]
pub use self::query::{query, query_opt};

#[doc(inline)]
#[allow(deprecated)]
pub use self::path::{param, params, segment, PathExt};
