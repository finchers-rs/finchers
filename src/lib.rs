#![feature(rust_2018_preview)]
#![feature(tool_lints)] // for clippy

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
//! use finchers::path;
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
//! # std::mem::drop(move || {
//!     finchers::launch(post_api)
//!         .start("127.0.0.1:4000")
//! # });
//! }
//! ```

#![doc(
    html_root_url = "https://docs.rs/finchers/0.12.0-alpha.5",
    test(attr(feature(rust_2018_preview))),
)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
#![allow(keyword_idents)] // serde-rs/serde#1385
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", doc(test(attr(deny(warnings)))))]

extern crate bitflags;
extern crate bytes;
extern crate cookie;
extern crate either;
extern crate failure;
extern crate futures; // 0.1
extern crate http;
extern crate hyper;
extern crate hyperx;
extern crate log;
extern crate mime;
extern crate mime_guess;
extern crate percent_encoding;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
extern crate time;
extern crate tokio;
extern crate url;

#[cfg(test)]
extern crate matches;

#[macro_use]
mod macros;
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
pub use crate::launcher::launch;

/// A prelude for crates using the `finchers` crate.
pub mod prelude {
    pub use crate::endpoint;
    pub use crate::endpoint::wrapper::{EndpointWrapExt, Wrapper};
    pub use crate::endpoint::{Endpoint, IntoEndpoint, IntoEndpointExt, SendEndpoint};
    pub use crate::endpoints;
    pub use crate::error::HttpError;
}
