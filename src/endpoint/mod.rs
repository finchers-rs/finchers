//! The definition of `Endpoint` layer

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
pub use self::context::EndpointContext;
pub use self::endpoint::{Endpoint, IntoEndpoint};
pub use self::error::EndpointError;
pub use self::header::{header, header_opt};
pub use self::query::{query, query_opt};
pub use self::path::{param, params, segment};
