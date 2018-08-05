#![cfg_attr(feature = "nightly", feature(try_trait))]

//! Core primitives for constructing asynchronous HTTP services

#![doc(html_root_url = "https://docs.rs/finchers-core/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate bytes;
extern crate either;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate http;
extern crate mime;
extern crate percent_encoding;

#[cfg(feature = "hyper")]
extern crate hyper;

#[macro_use]
mod poll;

mod never;
mod option;
mod result;

pub mod endpoint;
pub mod error;
pub mod input;
pub mod output;
pub mod task;

// re-exports
pub use endpoint::Endpoint;
pub use error::{Error, HttpError};
pub use input::Input;
pub use never::Never;
pub use option::IsOption;
pub use output::{Output, Responder};
pub use poll::{Poll, PollResult};
pub use result::IsResult;
pub use task::Task;
