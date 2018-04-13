extern crate finchers_core;

#[macro_use]
extern crate futures;
extern crate http;
extern crate hyper;
extern crate tokio;
#[macro_use]
extern crate structopt;

pub mod config;
pub mod server;
pub mod service;

pub use config::Config;
pub use server::Server;
pub use service::{EndpointService, ErrorHandler, HttpService};

use finchers_core::endpoint::Endpoint;
use finchers_core::output::Responder;

pub fn run<E>(endpoint: E)
where
    E: Endpoint + Send + Sync + 'static,
    E::Item: Responder,
{
    let service = EndpointService::new(endpoint);
    let config = Config::from_env();
    Server::new(service, config).run();
}
