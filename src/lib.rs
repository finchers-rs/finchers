//! A combinator library for building asynchronous HTTP services.
//!
//! The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).
//!
//! # Features
//!
//! * Building an HTTP service by *combining* the primitive components
//! * Type-safe routing
//! * Asynchronous handling based on Futures and Hyper 0.11
//! * Focuses on stable channel
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
//! ```toml
//! [dependencies]
//! finchers = { git = "https://github.com/finchers-rs/finchers.git" }
//! ```
//!
//! ```rust
//! #[macro_use]
//! extern crate finchers;
//!
//! use finchers::Endpoint;
//! use finchers::output::Debug;
//!
//! fn build_endpoint() -> impl Endpoint<Item = Debug> + Send + Sync + 'static {
//!     use finchers::endpoint::prelude::*;
//!
//!     path("api/v1").right(choice![
//!         get(param())
//!             .map(|id: u64| format!("GET: id={}", id)),
//!         post(body())
//!             .map(|data: String| format!("POST: body={}", data)),
//!     ])
//!     .map(|val| Debug::new(val).pretty(true))
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
extern crate finchers_json;
extern crate finchers_runtime;
extern crate finchers_urlencoded;

pub extern crate futures;
pub extern crate http;
pub extern crate mime;

#[doc(hidden)]
pub use finchers_derive::*;

pub use finchers_core::{error, output};

pub mod endpoint {
    pub use finchers_core::endpoint::{task, Endpoint, IntoEndpoint, Task};
    pub use finchers_endpoint::{all, ok, EndpointExt};

    pub use finchers_http::{body, header, method, path, FromBody, FromHeader, FromSegment, FromSegments};

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
        pub use finchers_endpoint::EndpointExt;
        pub use finchers_http::body::{body, body_stream};
        pub use finchers_http::header::header;
        pub use finchers_http::method::{delete, get, head, patch, post, put};
        pub use finchers_http::path::{param, params, path};
    }
}

pub mod input {
    pub use finchers_core::input::{Body, BodyStream, Error, ErrorKind, Input};
}

pub mod runtime {
    pub use finchers_runtime::{EndpointServiceExt, FinchersService, FinchersServiceFuture, HttpService, Server};
}

pub mod json {
    pub use finchers_json::{Error, Json};
}

pub mod urlencoded {
    pub use finchers_urlencoded::{from_csv, queries, queries_opt, queries_req, Error, Form, Queries, QueriesOptional,
                                  QueriesRequired};
}

pub use finchers_core::endpoint::Endpoint;
pub use finchers_core::output::Responder;
pub use finchers_core::{Input, Output};
pub use finchers_endpoint::EndpointExt;

#[macro_use]
mod macros;

pub fn run<E>(endpoint: E)
where
    E: Endpoint + Send + Sync + 'static,
    E::Item: Responder,
{
    use finchers_runtime::{EndpointServiceExt, Server};
    let service = endpoint.into_service();
    Server::new(service).run();
}
