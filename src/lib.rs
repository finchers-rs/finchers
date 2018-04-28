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
//! fn build_endpoint() -> impl Endpoint<Output = String> + Send + Sync + 'static {
//!     use finchers::endpoint::prelude::*;
//!
//!     path("api/v1").right(choice![
//!         get(param())
//!             .map(|id: u64| format!("GET: id={}", id)),
//!         post(data())
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

extern crate finchers_core;
#[allow(unused_imports)]
#[macro_use]
extern crate finchers_derive;
extern crate finchers_endpoint;
extern crate finchers_http;
extern crate finchers_runtime;

pub extern crate futures;
pub extern crate http;
pub extern crate mime;

#[doc(hidden)]
pub use finchers_derive::*;

pub mod error {
    pub use finchers_core::error::{BadRequest, HttpError, ServerError};
}

pub mod endpoint {
    pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
    pub use finchers_endpoint::{abort, all, just, lazy, EndpointExt, EndpointOptionExt, EndpointResultExt};
    pub use finchers_http::{body, header, method, path, query, FromData, FromHeader, FromSegment, FromSegments};

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
        pub use finchers_endpoint::{EndpointExt, EndpointOptionExt, EndpointResultExt};
        pub use finchers_http::body::{data, raw_body};
        pub use finchers_http::header::header;
        pub use finchers_http::method::{delete, get, head, patch, post, put};
        pub use finchers_http::path::{param, params, path};
    }
}

pub mod input {
    pub use finchers_core::input::{Data, Error, ErrorKind, Input, RequestBody};
}

pub mod output {
    pub use finchers_core::output::{Body, Debug, HttpStatus, Responder};
}

pub mod runtime {
    pub use finchers_runtime::server::Server;
    pub use finchers_runtime::service::{EndpointService, HttpService};
}

pub use finchers_core::{Endpoint, HttpError, Input, Output, Responder};
pub use finchers_endpoint::{EndpointExt, EndpointResultExt};
pub use finchers_http::json::Json;

#[macro_use]
mod macros;

pub use finchers_runtime::run;
