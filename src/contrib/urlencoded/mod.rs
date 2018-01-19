//! Support for parsing urlencoded queries and message body
//!
//! Provided endpoins/utilities are as follows:
//!
//! * `Queries<T>` - Parses the query string in incoming request to the value of `T`, otherwise skips the current route.
//! * `QueriesRequired<T>` - Similar to `Queries`, but always matches and returns an error if the query is missing.
//! * `QueriesOptional<T>` - Similar to `Queries`, but always matches and returns an `Option<T>`.
//! * `Form<T>` - Represents a type deserialized from an urlencoded request body.
//!
//! # Examples
//!
//! ```ignore
//! struct Param {
//!     name: String,
//!     required: bool,
//! }
//!
//! impl FromUrlEncoded for Param { ... }
//!
//! let endpoint = queries_req::<Param>();
//!
//! let endpoint = get(queries().map_err(Into::into))
//!     .or(post(body().map(|Form(body)| body)).map_err(Into::into))
//!     .and_then(|param| { ... });
//! ```

pub mod simple;
pub mod serde;

pub use self::simple::*;
