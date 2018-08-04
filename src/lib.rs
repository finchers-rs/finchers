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
//! ```rust
//! #[macro_use]
//! extern crate finchers;
//!
//! use finchers::Endpoint;
//!
//! fn build_endpoint() -> impl Endpoint<Output = String> + 'static {
//!     use finchers::endpoint::prelude::*;
//!
//!     path("api/v1").right(choice![
//!         get(param().unwrap_ok())
//!             .map(|id: u64| format!("GET: id={}", id)),
//!         post(body().unwrap_ok())
//!             .map(|data: String| format!("POST: body={}", data)),
//!     ])
//! }
//!
//! fn main() {
//!     let endpoint = build_endpoint();
//!
//! # std::thread::spawn(move || {
//!     finchers::run(endpoint);
//! # });
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers/0.11.0")]

extern crate finchers_core;
#[allow(unused_imports)]
#[macro_use]
extern crate finchers_derive;
extern crate finchers_ext;
extern crate finchers_http;
extern crate finchers_runtime;

#[doc(hidden)]
pub use finchers_derive::*;

pub mod error {
    pub use finchers_core::error::{BadRequest, HttpError, ServerError};
}

pub mod endpoint {
    pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
    pub use finchers_ext::{
        abort, all, just, lazy, EndpointExt, EndpointOptionExt, EndpointResultExt,
    };
    pub use finchers_http::{
        body, header, method, path, query, FromBody, FromHeader, FromSegment, FromSegments,
    };

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
        pub use finchers_ext::{EndpointExt, EndpointOptionExt, EndpointResultExt};
        pub use finchers_http::body::{body, raw_body};
        pub use finchers_http::header::header;
        pub use finchers_http::method::{delete, get, head, patch, post, put};
        pub use finchers_http::path::{param, params, path};
    }
}

pub mod input {
    pub use finchers_core::input::{Data, Input, RequestBody};
}

pub mod output {
    pub use finchers_core::output::{Debug, HttpResponse, Responder, ResponseBody};
}

pub mod runtime {
    pub use finchers_runtime::endpoint::NewEndpointService;
    pub use finchers_runtime::server::Server;
    pub use finchers_runtime::service::{HttpService, NewHttpService};
}

pub use finchers_core::{Endpoint, HttpError, Input, Never, Output, Responder};
pub use finchers_http::json::Json;
pub use finchers_runtime::run;

#[macro_use]
mod macros;
