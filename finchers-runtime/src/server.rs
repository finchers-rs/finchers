//! Components for managing HTTP server.

use failure::Fail;
use futures::{Async, Future, Poll, Stream};
use http;
use hyper;
use hyper::server::{self, Http};
use slog::{Drain, Level, Logger};
use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;
use tokio;
use tokio::net::TcpListener;
use {slog_async, slog_term};

use finchers_core::input::RequestBody;
use service::{HttpService, NewHttpService, Payload};

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

/// A builder for running the HTTP server based on given HTTP service and configuration.
#[derive(Debug)]
pub struct Server<S> {
    new_service: S,
    config: Config,
}

impl<S> Server<S>
where
    S: NewHttpService<RequestBody = RequestBody> + Send + Sync + 'static,
    S::ResponseBody: Payload + Send + 'static,
    <S::ResponseBody as Payload>::Data: Send,
    <S::ResponseBody as Payload>::Error: Into<hyper::Error>,
    S::Service: Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<hyper::Error>,
    <S::Service as HttpService>::Future: Send + 'static,
    S::InitError: Fail,
{
    /// Create a new launcher from given service.
    pub fn new(new_service: S, config: Config) -> Server<S> {
        Server {
            new_service,
            config,
        }
    }

    /// Start the HTTP server with given configurations
    #[inline]
    pub fn launch(self) {
        let Server {
            new_service,
            config,
        } = self;
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
        let incoming = listener.incoming().map_err({
            let logger = logger.clone();
            move |err| trace!(logger, "failed to accept a TCP connection: {}", err)
        });

        let server = incoming.for_each(move |stream| {
            let logger = logger.new(o! {
                "ip_addr" => stream.peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|_| "<error>".into()),
            });

            new_service
                .new_service()
                .map_err({
                    let logger = logger.clone();
                    move |e| error!(logger, "failed to create a new service: {}", e)
                })
                .and_then(move |service| {
                    let wrapped_service = WrappedService {
                        service: RefCell::new(service),
                        logger: logger.clone(),
                    };
                    let protocol = Http::<<S::ResponseBody as Payload>::Data>::new();
                    let conn = protocol.serve_connection(stream, wrapped_service);

                    conn.map_err(move |e| error!(logger, "during serving a connection: {}", e))
                })
        });

        tokio::run(server);
    }
}

scoped_thread_local!(static LOGGER: Logger);

/// Execute a closure with the reference to `Logger` associated with the current scope.
pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    LOGGER.with(|logger| f(logger))
}

#[derive(Debug)]
struct WrappedService<S> {
    // FIXME: remove RefCell
    service: RefCell<S>,
    logger: Logger,
}

impl<S> server::Service for WrappedService<S>
where
    S: HttpService<RequestBody = RequestBody>,
    S::Error: Into<hyper::Error> + 'static,
    S::ResponseBody: Payload,
    S::Future: Send + 'static,
{
    type Request = hyper::Request;
    type Response = hyper::Response<WrappedBody<S::ResponseBody>>;
    type Error = hyper::Error;
    type Future = WrappedServiceFuture<S::Future>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let request = http::Request::from(request).map(RequestBody::from_hyp);
        let logger = self.logger.new(o!{
            "method" => request.method().to_string(),
            "path" => request.uri().path().to_owned(),
        });

        let start = Instant::now();
        let future = LOGGER.set(&logger, || self.service.borrow_mut().call(request));
        WrappedServiceFuture {
            future,
            logger,
            start,
        }
    }
}

#[allow(missing_debug_implementations)]
struct WrappedServiceFuture<F> {
    future: F,
    logger: Logger,
    start: Instant,
}

impl<F, Bd> Future for WrappedServiceFuture<F>
where
    F: Future<Item = http::Response<Bd>>,
    Bd: Payload,
    F::Error: Into<hyper::Error> + 'static,
{
    type Item = hyper::Response<WrappedBody<Bd>>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = {
            let future = &mut self.future;
            try_ready!(LOGGER.set(&self.logger, || {
                future.poll().map_err(Into::<hyper::Error>::into)
            }))
        };
        let end = Instant::now();
        let duration = end - self.start;
        let duration_msec = duration.as_secs() * 10 + duration.subsec_nanos() as u64 / 1_000_000;
        info!(self.logger, "{} ({} ms)", response.status(), duration_msec);
        Ok(Async::Ready(hyper::Response::from(
            response.map(WrappedBody),
        )))
    }
}

#[derive(Debug)]
struct WrappedBody<Bd>(Bd);

impl<Bd: Payload> Stream for WrappedBody<Bd>
where
    Bd::Error: Into<hyper::Error>,
{
    type Item = Bd::Data;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll_data().map_err(Into::into)
    }
}
