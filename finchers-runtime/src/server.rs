#![allow(missing_docs)]

use failure;
use futures::Future;
use hyper::server::Server;
use slog::{Drain, Level, Logger};
use slog_async;
use slog_term;
use std::net::{IpAddr, SocketAddr};
use structopt::StructOpt;
use tokio;

use app::App;
use finchers_core::endpoint::Endpoint;

/// All kinds of logging mode of `Server`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Silent,
    Normal,
    Debug,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "finchers")]
struct Cli {
    /// The host of listener address.
    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    host: IpAddr,

    /// The port of listener address.
    #[structopt(short = "p", long = "port", default_value = "5000")]
    port: u16,

    /// Set to silent mode.
    #[structopt(short = "s", long = "silent")]
    silent: bool,

    /// Set to debug mode (implies silent=false).
    #[structopt(short = "d", long = "debug")]
    debug: bool,
}

/// A set of configurations used by the runtime.
#[derive(Debug)]
pub struct Config {
    addr: SocketAddr,
    mode: Mode,
    cli: Option<Cli>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            addr: ([127, 0, 0, 1], 5000).into(),
            mode: Mode::Normal,
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
        if cli.silent {
            self.mode = Mode::Silent;
        }
        if cli.debug {
            self.mode = Mode::Debug;
        }
        self.cli = Some(cli);
    }

    #[allow(missing_docs)]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    #[allow(missing_docs)]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Create an instance of "Logger" from the current configuration.
    pub fn logger(&self) -> Logger {
        let level = match self.mode {
            Mode::Silent => Level::Error,
            Mode::Normal => Level::Info,
            Mode::Debug => Level::Debug,
        };
        let drain = slog_term::term_full()
            .filter(move |record| record.level() <= level)
            .fuse();
        let async_drain = slog_async::Async::new(drain).build().fuse();

        Logger::root(async_drain, o!())
    }
}

pub type LaunchResult<T> = Result<T, failure::Error>;

/// Start the server with given endpoint and default configuration.
pub fn launch(endpoint: impl Endpoint) -> LaunchResult<()> {
    let config = Config::from_env();
    let logger = config.logger();

    let new_service = App::new(endpoint, logger.clone());
    let server = Server::try_bind(&config.addr())?
        .serve(new_service)
        .map_err({
            let logger = logger.clone();
            move |err| {
                error!(logger, "server error: {}", err);
            }
        });

    info!(logger, "Listening on http://{}", config.addr());
    tokio::run(server);

    Ok(())
}
