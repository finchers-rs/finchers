#![feature(rust_preview_2018)]
#![feature(use_extern_macros)]
#![feature(pin, arbitrary_self_types, futures_api)]

//! A combinator library for building asynchronous HTTP services.
//!
//! The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).
//!
//! # Features
//!
//! * Asynchronous handling powerd by futures and Tokio
//! * Building an HTTP service by *combining* the primitive components
//! * Type-safe routing without (unstable) procedural macros
//! * Focusing on stable channel
//!
//! # References
//!
//! * [User Guide][user-guide]
//! * [API documentation (released)][released-api]
//! * [API documentation (master)][master-api]
//!
//!
//! [user-guide]: https://finchers-rs.github.io/guide
//! [released-api]: https://docs.rs/finchers/*/finchers
//! [master-api]: https://finchers-rs.github.io/api/finchers/
//!
//! # Example
//!
//! ```ignore
//! #![feature(rust_2018_preview)]
//!
//! extern crate finchers;
//! extern crate futures_util;
//!
//! use finchers::Endpoint;
//! use futures_util::future::ready;
//!
//! fn build_endpoint() -> impl Endpoint {
//!     use finchers::endpoint::prelude::*;
//!     use finchers::macros::routes;
//!
//!     path("api/v1").and(routes![
//!         get(param())
//!             .and_then(|id: u64| ready(Ok((format!("GET: id={}", id),)))),
//!
//!         post(body())
//!             .and_then(|data: String| ready(Ok((format!("POST: body={}", data),)))),
//!     ])
//! }
//!
//! fn main() -> finchers::LaunchResult<()> {
//!     let endpoint = build_endpoint();
//!
//! # std::mem::drop(move || {
//!     finchers::launch(endpoint)
//! # });
//! # Ok(())
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers/0.11.0")]

extern crate finchers_core;
extern crate finchers_derive;

extern crate bytes;
extern crate futures;      // 0.1
extern crate futures_core; // 0.3
extern crate futures_util; // 0.3
extern crate http;
extern crate hyper;
#[macro_use]
extern crate structopt;
extern crate failure;
#[macro_use]
extern crate scoped_tls;
extern crate tokio;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

#[doc(hidden)]
pub use finchers_derive::*;

pub mod error {
    pub use finchers_core::error::{Error, Failure, HttpError};
}

pub mod endpoint {
    pub use finchers_core::endpoint::{ok, reject};
    pub use finchers_core::endpoint::{Endpoint, EndpointBase, IntoEndpoint};
    pub use finchers_core::endpoints::{body, header, method, path, query};

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_core::endpoint::EndpointExt;
        pub use finchers_core::endpoint::{Endpoint, EndpointBase, IntoEndpoint};
        pub use finchers_core::endpoints::body::{body, raw_body};
        pub use finchers_core::endpoints::header::header;
        pub use finchers_core::endpoints::method::{delete, get, head, patch, post, put};
        pub use finchers_core::endpoints::path::{param, path};
    }
}

pub mod input {
    pub use finchers_core::input::{
        Data, FromBody, FromHeader, FromQuery, FromSegment, Input, RequestBody,
    };
}

pub mod output {
    pub use finchers_core::output::{payloads, responders, Responder};
}

pub mod macros {
    pub use finchers_core::routes;
}

pub mod runtime;

pub use finchers_core::endpoint::{Endpoint, EndpointBase};
pub use finchers_core::error::{HttpError, Never};
pub use finchers_core::input::Input;
pub use finchers_core::json::{HttpResponse, Json};
pub use finchers_core::output::Responder;

pub use runtime::server::{launch, LaunchResult};

#[doc(hidden)]
pub mod _derive {
    pub use finchers_core::json::HttpResponse;
    pub use http::StatusCode;
}
