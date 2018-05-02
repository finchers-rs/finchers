use futures::{Future, Poll, Stream};
use http;
use hyper;
use hyper::server::{service_fn, Http};
use slog::{Drain, Level, Logger};
use std::cell::RefCell;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;
use tokio;
use tokio::net::TcpListener;
use {slog_async, slog_term};

use finchers_core::endpoint::Endpoint;
use finchers_core::input::RequestBody;
use finchers_core::output::Responder;
use service::{HttpService, NewHttpService};

use endpoint::NewEndpointService;

/// Start the server with given endpoint and default configuration.
pub fn run<E>(endpoint: E)
where
    E: Endpoint + 'static,
    E::Output: Responder,
{
    let config = Config::from_env();
    let new_service = NewEndpointService::new(endpoint);
    Server::new(new_service, config).run();
}

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

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

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

#[derive(Debug)]
pub struct Server<S> {
    new_service: S,
    config: Config,
}

impl<S> Server<S>
where
    S: NewHttpService<RequestBody = RequestBody> + Send + Sync + 'static,
    S::Service: Send + 'static,
    S::Future: Send + 'static,
    S::ResponseBody: Stream<Error = io::Error> + Send + 'static,
    <S::ResponseBody as Stream>::Item: AsRef<[u8]> + Send + 'static,
    S::Error: Into<hyper::Error>,
    <S::Service as HttpService>::Future: Send + 'static,
{
    /// Create a new launcher from given service.
    pub fn new(new_service: S, config: Config) -> Server<S> {
        Server { new_service, config }
    }

    /// Start the HTTP server with given configurations
    #[inline]
    pub fn run(self) {
        let Server { new_service, config } = self;
        let new_service = Arc::new(new_service);

        let logger = config.logger();
        info!(logger, "Listening on {}", config.addr());

        let listener = match TcpListener::bind(&config.addr()) {
            Ok(listener) => listener,
            Err(err) => {
                crit!(logger, "Failed to create TcpListener: {}", err);
                ::std::process::exit(1);
            }
        };

        let server = listener
            .incoming()
            .map_err({
                let logger = logger.clone();
                move |err| trace!(logger, "failed to accept: {}", err)
            })
            .for_each(move |stream| {
                let logger = logger.new(o! {
                    "ip_addr" => stream.peer_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|_| "<error>".into()),
                });

                // FIXME: move to the root.
                let protocol = Http::<<S::ResponseBody as Stream>::Item>::new();

                new_service
                    .new_service()
                    .map_err(|_e| eprintln!("TODO: log"))
                    .and_then(move |service| {
                        // FIXME: remove RefCell
                        let service = RefCell::new(service);

                        let service = service_fn(move |request: hyper::Request<hyper::Body>| {
                            let request = http::Request::from(request).map(RequestBody::from_hyp);

                            let logger = logger.new(o!{
                                "method" => request.method().to_string(),
                                "path" => request.uri().path().to_owned(),
                            });
                            let start = Instant::now();

                            service
                                .borrow_mut()
                                .call(request)
                                .map(|response| hyper::Response::from(response.map(BodyWrapper)))
                                .map_err(Into::into)
                                .inspect(move |response| {
                                    let end = Instant::now();
                                    let duration = end - start;
                                    info!(
                                        logger,
                                        "{} ({} ms)",
                                        response.status(),
                                        duration.as_secs() / 10 + duration.subsec_nanos() as u64 / 1_000_000,
                                    );
                                })
                        });
                        let conn = protocol.serve_connection(stream, service);
                        conn.map(|_conn| ()).map_err(|_| ())
                    })
            });

        tokio::run(server);
    }
}

#[derive(Debug)]
struct BodyWrapper<Bd>(Bd);

impl<Bd> Stream for BodyWrapper<Bd>
where
    Bd: Stream<Error = io::Error>,
    Bd::Item: AsRef<[u8]> + 'static,
{
    type Item = Bd::Item;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll().map_err(Into::into)
    }
}
