use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;

use futures::Stream;
use hyper::Chunk;
use hyper::server::Http;
use net2::TcpBuilder;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};

use endpoint::Endpoint;
use http::CookieManager;
use responder::IntoResponder;
use super::{EndpointService, NoRoute};

/// The factory of HTTP service
#[derive(Debug)]
pub struct ServerBuilder {
    addrs: Vec<SocketAddr>,
    num_workers: usize,
    proto: Http<Chunk>,
    secret_key: Option<Vec<u8>>,
    no_route: Option<NoRoute>,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        ServerBuilder {
            addrs: vec![],
            num_workers: 1,
            proto: Http::new(),
            secret_key: None,
            no_route: None,
        }
    }
}

impl ServerBuilder {
    /// Set the listener address of the service
    pub fn bind<S: ToSocketAddrs>(mut self, addr: S) -> Self {
        self.addrs.extend(addr.to_socket_addrs().unwrap());
        self
    }

    /// Set the number of worker threads
    pub fn num_workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }

    /// Set the "raw" HTTP protocol
    pub fn proto(mut self, proto: Http<Chunk>) -> Self {
        self.proto = proto;
        self
    }

    /// Set the secret key used by `CookieManager`.
    pub fn secret_key<K: Into<Vec<u8>>>(mut self, key: K) -> Self {
        self.secret_key = Some(key.into());
        self
    }

    /// Start an HTTP server with given endpoint
    pub fn serve<E>(mut self, endpoint: E)
    where
        E: Endpoint + Clone + Send + Sync + 'static,
        E::Item: IntoResponder,
        E::Error: IntoResponder,
    {
        if self.addrs.is_empty() {
            self.addrs.push("0.0.0.0:4000".parse().unwrap());
        }

        let cookie_manager = match self.secret_key {
            Some(key) => CookieManager::new(&key),
            None => CookieManager::default(),
        };

        let no_route = self.no_route.unwrap_or_default();

        let mut worker = Worker::new(endpoint, cookie_manager, no_route, self.proto, self.addrs);
        if self.num_workers > 1 {
            worker.reuse_port();
        }

        for _ in 0..(self.num_workers - 1) {
            let worker = worker.clone();
            thread::spawn(move || {
                let _ = worker.run();
            });
        }
        let _ = worker.run();
    }
}

/// The context of worker threads
#[derive(Debug, Clone)]
pub struct Worker<E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    endpoint: E,
    cookie_manager: CookieManager,
    no_route: NoRoute,
    proto: Http<Chunk>,
    addrs: Vec<SocketAddr>,
    capacity: i32,
    reuse_port: bool,
}

impl<E> Worker<E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    #[allow(missing_docs)]
    pub fn new(
        endpoint: E,
        cookie_manager: CookieManager,
        no_route: NoRoute,
        proto: Http<Chunk>,
        addrs: Vec<SocketAddr>,
    ) -> Self {
        Worker {
            endpoint,
            cookie_manager,
            no_route,
            proto,
            addrs,
            reuse_port: false,
            capacity: 1024,
        }
    }

    #[allow(missing_docs)]
    pub fn reuse_port(&mut self) {
        self.reuse_port = true;
    }

    #[allow(missing_docs)]
    pub fn capacity(&mut self, capacity: i32) {
        self.capacity = capacity;
    }

    #[allow(missing_docs)]
    pub fn run(&self) -> io::Result<()> {
        let mut core = Core::new()?;
        let handle = core.handle();
        let service = EndpointService {
            endpoint: self.endpoint.clone(),
            handle: handle.clone(),
            cookie_manager: self.cookie_manager.clone(),
            no_route: self.no_route.clone(),
        };

        let server = self.build_listener(&handle)?
            .incoming()
            .for_each(|(sock, addr)| {
                self.proto
                    .bind_connection(&handle, sock, addr, service.clone());
                Ok(())
            });

        core.run(server)
    }

    fn build_listener(&self, handle: &Handle) -> io::Result<TcpListener> {
        // TODO: bind to multiple listener addresses.
        let addr = &self.addrs[0];

        let listener = match *addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4()?,
            SocketAddr::V6(..) => TcpBuilder::new_v6()?,
        };

        listener.reuse_address(true)?;
        if self.reuse_port {
            reuse_port(&listener)?;
        }

        listener.bind(addr)?;
        let l = listener.listen(self.capacity)?;

        TcpListener::from_listener(l, addr, handle)
    }
}

#[cfg(not(windows))]
fn reuse_port(tcp: &TcpBuilder) -> io::Result<()> {
    use net2::unix::UnixTcpBuilderExt;
    tcp.reuse_port(true).map(|_| ())
}

#[cfg(windows)]
fn reuse_port(_: &TcpBuilder) -> io::Result<()> {
    Ok(())
}
