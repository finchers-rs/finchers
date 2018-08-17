//! Runtime support for Finchers, which supports serving asynchronous HTTP services.

pub mod local;

use failure;
use futures::Future;
use hyper::server::Server;
use log::{error, info, log};
use std::net::{IpAddr, SocketAddr};
use structopt::StructOpt;
use tokio;

use crate::app::App;
use crate::endpoint::Endpoint;
use crate::output::Responder;

#[derive(Debug, StructOpt)]
#[structopt(name = "finchers")]
struct Cli {
    /// The host of listener address.
    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    host: IpAddr,

    /// The port of listener address.
    #[structopt(short = "p", long = "port", default_value = "5000")]
    port: u16,
}

/// A set of configurations used by the runtime.
#[derive(Debug)]
pub struct Config {
    addr: SocketAddr,
    cli: Option<Cli>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            addr: ([127, 0, 0, 1], 5000).into(),
            cli: None,
        }
    }
}

impl Config {
    /// Create an instance of "Config" from the environment.
    pub fn from_env() -> Config {
        let mut config = Config::default();
        config.overwite_cli(Cli::from_args());
        config
    }

    fn overwite_cli(&mut self, cli: Cli) {
        self.addr = (cli.host, cli.port).into();
        self.cli = Some(cli);
    }

    #[allow(missing_docs)]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[allow(missing_docs)]
pub type LaunchResult<T> = Result<T, failure::Error>;

/// Start the server with given endpoint and default configuration.
pub fn launch<E>(endpoint: E) -> LaunchResult<()>
where
    E: Endpoint + Send + Sync + 'static,
    E::Output: Responder,
    E::Future: Send + 'static,
{
    let config = Config::from_env();

    let endpoint: &'static _ = unsafe { &*(&endpoint as *const _) };
    let new_service = App::new(endpoint);

    let server = Server::try_bind(&config.addr())?
        .serve(new_service)
        .map_err({
            move |err| {
                error!("server error: {}", err);
            }
        });

    info!("Listening on http://{}", config.addr());
    tokio::run(server);

    Ok(())
}
