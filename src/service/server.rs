//! Definition of HTTP services for Hyper

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;

use futures::Stream;
use hyper::Chunk;
use hyper::server::Http;
use net2::TcpBuilder;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};

use endpoint::{Endpoint, NotFound};
use responder::IntoResponder;

/// The factory of HTTP service
#[derive(Debug)]
pub struct ServerBuilder {
    addrs: Vec<SocketAddr>,
    num_workers: usize,
    proto: Http<Chunk>,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        ServerBuilder {
            addrs: vec![],
            num_workers: 1,
            proto: Http::new(),
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

    /// Start an HTTP server with given endpoint
    pub fn serve<E>(mut self, endpoint: E)
    where
        E: Endpoint + Clone + Send + Sync + 'static,
        E::Item: IntoResponder,
        E::Error: IntoResponder + From<NotFound>,
    {
        if self.addrs.is_empty() {
            self.addrs.push("0.0.0.0:4000".parse().unwrap());
        }

        let mut worker = Worker::new(endpoint, self.proto, self.addrs);
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

#[derive(Debug, Clone)]
pub struct Worker<E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    endpoint: E,
    proto: Http<Chunk>,
    addrs: Vec<SocketAddr>,
    capacity: i32,
    reuse_port: bool,
}

impl<E> Worker<E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<NotFound>,
{
    pub fn new(endpoint: E, proto: Http<Chunk>, addrs: Vec<SocketAddr>) -> Self {
        Worker {
            endpoint,
            proto,
            addrs,
            reuse_port: false,
            capacity: 1024,
        }
    }

    pub fn reuse_port(&mut self) {
        self.reuse_port = true;
    }

    pub fn capacity(&mut self, capacity: i32) {
        self.capacity = capacity;
    }

    pub fn run(&self) -> io::Result<()> {
        let mut core = Core::new()?;
        let handle = core.handle();

        let server = self.build_listener(&handle)?
            .incoming()
            .for_each(|(sock, addr)| {
                let service = self.endpoint.to_service(&handle);
                self.proto.bind_connection(&handle, sock, addr, service);
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
