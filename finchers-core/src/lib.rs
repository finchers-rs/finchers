#![cfg_attr(feature = "nightly", feature(try_trait))]
#![feature(rust_2018_preview)]

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
pub use crate::endpoint::Endpoint;
pub use crate::error::{Error, HttpError};
pub use crate::input::Input;
pub use crate::never::Never;
pub use crate::option::IsOption;
pub use crate::output::{Output, Responder};
pub use crate::poll::{Poll, PollResult};
pub use crate::result::IsResult;
pub use crate::task::Task;
