//! A combinator library for building asynchronous HTTP services.
//!
//! The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).
//!
//! # Features
//!
//! * Asynchronous handling powerd by futures and Tokio
//! * Building an HTTP service by *combining* the primitive components
//! * Type-safe routing without (unstable) procedural macros
//!
//! # Example
//!
//! ```
//! use finchers::prelude::*;
//! use finchers::endpoint::syntax::path;
//!
//! # fn main() -> izanami::Result<()> {
//! let get_post = path!(@get "/<u64>")
//!     .map(|id: u64| format!("GET: id={}", id));
//!
//! let create_post = path!(@post "/")
//!     .and(endpoints::body::text())
//!     .map(|data: String| format!("POST: body={}", data));
//!
//! let endpoint = path!("/posts")
//!     .and(get_post.or(create_post));
//!
//! # drop(move || -> izanami::Result<_> {
//! izanami::Server::bind_tcp(&"127.0.0.1:4000".parse()?)?
//!     .start(endpoint.into_service())
//! # });
//! # Ok(())
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers/0.14.0-dev")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    unused
)]
#![forbid(clippy::unimplemented)]
#![doc(test(attr(deny(warnings))))]

mod common;

pub mod action;
pub mod endpoint;
pub mod endpoints;
pub mod error;
pub mod output;
pub mod service;
pub mod test;
pub mod util;

/// A prelude for crates using the `finchers` crate.
pub mod prelude {
    pub use crate::endpoint;
    pub use crate::endpoint::{Endpoint, EndpointExt};
    pub use crate::endpoints;
    pub use crate::error::HttpError;
    pub use crate::service::EndpointServiceExt;
}
