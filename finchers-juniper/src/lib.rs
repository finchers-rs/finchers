// FIXME: remove this feature gate as soon as the rustc version used in docs.rs is updated
#![cfg_attr(finchers_inject_extern_prelude, feature(extern_prelude))]

//! A set of extensions for supporting Juniper integration.
//!
//! # Examples
//!
//! ```
//! #[macro_use]
//! extern crate finchers;
//! # use finchers::prelude::*;
//! extern crate finchers_juniper;
//! #[macro_use]
//! extern crate juniper;
//!
//! use juniper::{EmptyMutation, RootNode};
//!
//! // The contextual information used when GraphQL query executes.
//! //
//! // Typically it contains a connection retrieved from pool
//! // or credential information extracted from HTTP headers.
//! struct MyContext {
//!     // ...
//! }
//! impl juniper::Context for MyContext {}
//!
//! struct Query {}
//! graphql_object!(Query: MyContext |&self| {
//!     field apiVersion() -> &str { "1.0" }
//!     // ...
//! });
//!
//! # fn main() {
//! let schema = RootNode::new(
//!     Query {},
//!     EmptyMutation::<MyContext>::new(),
//! );
//!
//! // An endpoint which acquires a GraphQL context from request.
//! let fetch_graphql_context =
//!     endpoint::unit().map(|| MyContext { /* ... */ });
//!
//! // Build an endpoint which handles GraphQL requests.
//! let endpoint = path!(@get / "graphql" /)
//!     .and(fetch_graphql_context)
//!     .wrap(finchers_juniper::execute::nonblocking(schema));
//! # drop(move || {
//! # finchers::server::start(endpoint).serve("127.0.0.1:4000")
//! # });
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers-juniper/0.2.1")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
// #![warn(rust_2018_compatibility)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, doc(test(attr(deny(warnings)))))]

extern crate bytes;
extern crate failure;
extern crate finchers;
#[macro_use]
extern crate futures;
extern crate juniper;
#[macro_use]
extern crate log;
extern crate percent_encoding;
#[macro_use]
extern crate serde;
extern crate http;
extern crate serde_json;
extern crate serde_qs;

#[cfg(test)]
#[macro_use]
extern crate matches;

pub mod execute;
pub mod graphiql;
pub mod request;

pub use graphiql::graphiql_source;
pub use request::graphql_request;
