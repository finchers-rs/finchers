use futures::{Future, Poll, Stream};
use http;
use hyper;
use hyper::server::{service_fn, Http};
use service::HttpService;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio;
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct Server<S> {
    service: S,
    addr: Option<SocketAddr>,
}

impl<S> Server<S>
where
    S: HttpService + Send + Sync + 'static,
    S::RequestBody: From<hyper::Body>,
    S::Future: Send,
    S::ResponseBody: Stream<Error = io::Error> + Send + 'static,
    <S::ResponseBody as Stream>::Item: AsRef<[u8]> + Send + 'static,
{
    /// Create a new launcher from given service.
    pub fn new(service: S) -> Self {
        Server { service, addr: None }
    }

    pub fn bind<T: ToSocketAddrs>(mut self, addr: T) -> Self {
        self.addr = addr.to_socket_addrs().unwrap().next();
        self
    }

    /// Start the HTTP server with given configurations
    #[inline]
    pub fn run(self) {
        let Server { service, addr } = self;
        let service = Arc::new(service);
        let addr = addr.unwrap_or_else(|| ([127, 0, 0, 1], 4000).into());

        let listener = TcpListener::bind(&addr).expect("failed to create TcpListener");
        let protocol = Http::<<S::ResponseBody as Stream>::Item>::new();
        let server = listener
            .incoming()
            .map_err(|err| eprintln!("failed to accept: {}", err))
            .for_each(move |stream| {
                let service = service.clone();
                let service = service_fn(move |request| {
                    let request = http::Request::from(request).map(Into::into);
                    service
                        .call(request)
                        .map(|response| hyper::Response::from(response.map(BodyWrapper)))
                        .map_err(hyper::Error::from)
                });
                let conn = protocol.serve_connection(stream, service);
                conn.map(|_conn| ()).map_err(|_| ())
            });
        tokio::run(server);
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BodyWrapper<Bd>(Bd);

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
