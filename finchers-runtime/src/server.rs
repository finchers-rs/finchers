use futures::{Future, Poll, Stream};
use http;
use hyper;
use hyper::server::{service_fn, Http};
use std::io;
use std::sync::Arc;
use std::time::Instant;
use tokio;
use tokio::net::TcpListener;

use Config;
use service::HttpService;

#[derive(Debug)]
pub struct Server<S> {
    service: S,
    config: Config,
}

impl<S> Server<S>
where
    S: HttpService + Send + Sync + 'static,
    S::RequestBody: From<hyper::Body>,
    S::ResponseBody: Stream<Error = io::Error> + Send + 'static,
    <S::ResponseBody as Stream>::Item: AsRef<[u8]> + Send + 'static,
    S::Error: Into<hyper::Error>,
    S::Future: Send,
{
    /// Create a new launcher from given service.
    pub fn new(service: S, config: Config) -> Server<S> {
        Server { service, config }
    }

    /// Start the HTTP server with given configurations
    #[inline]
    pub fn run(self) {
        let Server { service, config } = self;
        let service = Arc::new(service);

        let logger = config.logger();
        info!(logger, "Listening on {}", config.addr());

        let listener = match TcpListener::bind(&config.addr()) {
            Ok(listener) => listener,
            Err(err) => {
                crit!(logger, "Failed to create TcpListener: {}", err);
                ::std::process::exit(1);
            }
        };

        let protocol = Http::<<S::ResponseBody as Stream>::Item>::new();
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

                let service = service.clone();
                let service = service_fn(move |request: hyper::Request<_>| {
                    let request = http::Request::from(request).map(Into::into);

                    let logger = logger.new(o!{
                        "method" => request.method().to_string(),
                        "path" => request.uri().path().to_owned(),
                    });
                    let start = Instant::now();

                    service
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
