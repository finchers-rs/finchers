#![feature(rust_2018_preview)]
#![feature(use_extern_macros)]
#![feature(pin, arbitrary_self_types, futures_api)]

//! Core primitives for constructing asynchronous HTTP services

#![doc(html_root_url = "https://docs.rs/finchers-core/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
mod macros;

pub mod either;
pub mod endpoint;
pub mod error;
pub mod generic;
pub mod endpoints;
pub mod input;
pub mod output;
pub mod test;
