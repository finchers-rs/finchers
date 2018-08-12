#![feature(rust_2018_preview)]
#![feature(use_extern_macros)]
#![feature(pin, arbitrary_self_types, futures_api)]

//! Core primitives for constructing asynchronous HTTP services

#![doc(html_root_url = "https://docs.rs/finchers-core/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate bytes;
extern crate failure;
extern crate futures;
extern crate futures_core;
extern crate futures_executor;
extern crate futures_util;
extern crate http;
extern crate hyper;
extern crate mime;
extern crate percent_encoding;
extern crate pin_utils;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;

#[macro_use]
mod macros;

pub mod either;
pub mod endpoint;
pub mod endpoints;
pub mod error;
pub mod generic;
pub mod input;
pub mod json;
pub mod local;
pub mod output;
