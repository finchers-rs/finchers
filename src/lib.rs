// FIXME: remove this feature gate as soon as the rustc version used in docs.rs is updated
#![cfg_attr(finchers_inject_extern_prelude, feature(extern_prelude))]

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
//! ```no_run
//! #[macro_use]
//! extern crate finchers;
//!
//! use finchers::prelude::*;
//!
//! fn main() {
//!     let get_post = path!(@get / u64 /)
//!         .map(|id: u64| format!("GET: id={}", id));
//!
//!     let create_post = path!(@post /)
//!         .and(endpoints::body::text())
//!         .map(|data: String| format!("POST: body={}", data));
//!
//!     let post_api = path!(/ "posts")
//!         .and(get_post.or(create_post));
//!
//!     finchers::launch(post_api)
//!         .start("127.0.0.1:4000");
//! }
//! ```

// master
//#![doc(html_root_url = "https://finchers-rs.github.io/finchers/")]
// released
#![doc(html_root_url = "https://finchers-rs.github.io/docs/finchers/v0.12.0")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
// FIXME: re-enable the following lint after shipping rust-1.31 out
// #![warn(rust_2018_compatibility)]
#![cfg_attr(finchers_deny_warnings, deny(warnings))]
#![cfg_attr(finchers_deny_warnings, doc(test(attr(deny(warnings)))))]

#[macro_use]
extern crate bitflags;
extern crate bytes;
extern crate cookie;
extern crate either;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures;
extern crate http;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate mime_guess;
#[macro_use]
extern crate percent_encoding;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
extern crate tokio;
extern crate url;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod app;
mod common;

pub mod endpoint;
pub mod endpoints;
pub mod error;
pub mod input;
pub mod launcher;
pub mod local;
pub mod output;

#[doc(inline)]
pub use launcher::launch;

/// A prelude for crates using the `finchers` crate.
pub mod prelude {
    pub use endpoint;
    pub use endpoint::wrapper::{EndpointWrapExt, Wrapper};
    pub use endpoint::{Endpoint, IntoEndpoint, IntoEndpointExt, SendEndpoint};
    pub use endpoints;
    pub use error::HttpError;
}
