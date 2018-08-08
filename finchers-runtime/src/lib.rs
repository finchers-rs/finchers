//! Runtime support for Finchers, which supports serving asynchronous HTTP services.

#![doc(html_root_url = "https://docs.rs/finchers-runtime/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate finchers_core;

extern crate bytes;
#[macro_use]
extern crate futures;
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

mod apply;
pub mod endpoint;
pub mod local;
pub mod server;
pub mod service;

pub use server::{Config, Server};
pub use service::{HttpService, NewHttpService};

use finchers_core::endpoint::Endpoint;

/// Start the server with given endpoint and default configuration.
pub fn run<E: Endpoint>(endpoint: E) {
    let config = Config::from_env();
    let new_service = endpoint::NewEndpointService::new(endpoint);
    Server::new(new_service, config).launch();
}
