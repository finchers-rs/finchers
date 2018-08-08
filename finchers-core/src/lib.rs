#![feature(rust_2018_preview)]
#![feature(use_extern_macros)]

//! Core primitives for constructing asynchronous HTTP services

#![doc(html_root_url = "https://docs.rs/finchers-core/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
pub mod future;

#[macro_use]
mod macros;

mod never;

pub mod endpoint;
pub mod error;
pub mod http;
pub mod input;
pub mod output;

// re-exports
pub use crate::endpoint::Endpoint;
pub use crate::error::{Error, HttpError};
pub use crate::input::Input;
pub use crate::never::Never;
pub use crate::output::{Output, Responder};
