//! Built-in endpoints.

pub mod body;
pub mod cookie;
pub mod fs;
pub mod header;
#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.4",
    note = "use components in `endpoint::syntax` instead"
)]
pub mod method;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.4",
    note = "use components in `endpoint::syntax` instead"
)]
pub mod path;
pub mod query;
