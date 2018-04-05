//! A combinator library for building asynchronous HTTP services.

#![doc(html_root_url = "https://docs.rs/finchers/0.10.1")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate bytes;
#[macro_use]
extern crate futures;
extern crate http;
extern crate hyper;
extern crate mime;

pub mod error;
pub mod never;
pub mod request;
pub mod response;
