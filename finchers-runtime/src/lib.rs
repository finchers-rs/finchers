#![feature(pin, arbitrary_self_types, futures_api)]

//! Runtime support for Finchers, which supports serving asynchronous HTTP services.

#![doc(html_root_url = "https://docs.rs/finchers-runtime/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate finchers_core;

extern crate bytes;
extern crate futures;      // 0.1
extern crate futures_core; // 0.3
extern crate futures_util; // 0.3
extern crate http;
extern crate hyper;
#[macro_use]
extern crate structopt;
extern crate failure;
#[macro_use]
extern crate scoped_tls;
extern crate tokio;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

pub mod app;
pub mod server;
