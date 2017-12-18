//! Definition of the trait `Endpoint` and its implementors

mod body;
mod context;
mod endpoint;
mod error;
mod header;
pub mod method;
mod path;
pub mod primitive;
mod query;

pub use self::body::body;
pub use self::endpoint::{Endpoint, IntoEndpoint};
pub use self::error::EndpointError;
pub use self::context::{EndpointContext, FromParam};
pub use self::header::{header, header_opt};
pub use self::method::MatchMethod;
pub use self::query::{query, query_opt};
pub use self::path::{param, params, segment};
