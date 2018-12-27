// FIXME: remove it as soon as the rustc version used in docs.rs is updated
#![cfg_attr(finchers_inject_extern_prelude, feature(extern_prelude))]

//! Session support for Finchers.
//!
//! Supported backends:
//!
//! * Cookie
//! * In-memory database
//! * Redis (requires the feature flag `feature = "redis"`)
//!
//! # Feature Flags
//!
//! * `redis` - enable Redis backend (default: off)
//! * `secure` - enable signing and encryption support for Cookie values
//!              (default: on. it adds the crate `ring` to dependencies).

#![doc(html_root_url = "https://docs.rs/finchers-session/0.2.0")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
//#![warn(rust_2018_compatibility)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, doc(test(attr(deny(warnings)))))]

#[macro_use]
extern crate failure;
extern crate finchers;
#[cfg_attr(feature = "redis", macro_use)]
extern crate futures;
extern crate time;
extern crate uuid;

#[cfg(test)]
extern crate http;

mod session;
#[cfg(test)]
mod tests;
mod util;

pub mod cookie;
pub mod in_memory;
#[cfg(feature = "redis")]
pub mod redis;

pub use self::session::{RawSession, Session};
