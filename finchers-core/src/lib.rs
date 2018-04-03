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
#[cfg(feature = "tls")]
extern crate native_tls;
extern crate net2;
extern crate num_cpus;
extern crate tokio_core;
extern crate tokio_io;
#[cfg(feature = "tls")]
extern crate tokio_tls;

pub mod endpoint;
pub mod errors;
pub mod request;
pub mod response;
pub mod service;
pub mod test;
