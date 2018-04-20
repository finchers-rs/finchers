#![doc(html_url = "https://docs.rs/finchers-runtime/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(warnings)]

extern crate finchers_core;

extern crate futures;
extern crate http;
extern crate hyper;
extern crate tokio;
#[macro_use]
extern crate structopt;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

pub mod config;
pub mod server;
pub mod service;

pub use config::Config;
pub use launcher::run;
pub use server::Server;
pub use service::{EndpointService, ErrorHandler, HttpService};

mod launcher {
    use config::Config;
    use server::Server;
    use service::EndpointService;

    use finchers_core::endpoint::Endpoint;
    use finchers_core::output::Responder;

    /// Start the server with given endpoint and default configuration.
    pub fn run<E>(endpoint: E)
    where
        E: Endpoint + Send + Sync + 'static,
        E::Item: Responder,
    {
        run_with_config(endpoint, Config::from_env());
    }

    /// Start the server with given endpoint and given configuration.
    pub fn run_with_config<E>(endpoint: E, config: Config)
    where
        E: Endpoint + Send + Sync + 'static,
        E::Item: Responder,
    {
        let service = EndpointService::new(endpoint);
        Server::new(service, config).run();
    }
}
