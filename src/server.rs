//! Definition of HTTP services for Hyper

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use futures::Stream;
use hyper::Chunk;
use hyper::server::Http;
use net2::TcpBuilder;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};

use endpoint::Endpoint;
use response::Responder;
use service::EndpointService;


/// The factory of HTTP service
#[derive(Debug)]
pub struct Server<E> {
    endpoint: E,
    addr: Option<String>,
    num_workers: Option<usize>,
}

impl<E: Endpoint> Server<E> {
    /// Create a new instance of `Server` from a `NewEndpoint`
    pub fn new(endpoint: E) -> Self {
        Self {
            endpoint,
            addr: None,
            num_workers: None,
        }
    }

    /// Set the listener address of the service
    pub fn bind<S: Into<String>>(mut self, addr: S) -> Self {
        self.addr = Some(addr.into());
        self
    }

    /// Set the number of worker threads
    pub fn num_workers(mut self, n: usize) -> Self {
        self.num_workers = Some(n);
        self
    }
}

impl<E> Server<E>
where
    E: Endpoint + Send + Sync + 'static,
    E::Item: Responder,
    E::Error: Responder,
{
    /// Start a HTTP server
    pub fn run_http(self) {
        let endpoint = Arc::new(self.endpoint);
        let proto = Http::new();
        let addr = self.addr.unwrap_or("0.0.0.0:4000".into()).parse().unwrap();
        let num_workers = self.num_workers.unwrap_or(1);

        for _ in 0..(num_workers - 1) {
            let endpoint = endpoint.clone();
            let proto = proto.clone();
            thread::spawn(move || {
                serve(endpoint, proto, num_workers, &addr);
            });
        }
        serve(endpoint, proto, num_workers, &addr);
    }
}

fn serve<E>(endpoint: E, proto: Http<Chunk>, num_workers: usize, addr: &SocketAddr)
where
    E: Endpoint + Clone + 'static,
    E::Item: Responder,
    E::Error: Responder,
{
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let listener = listener(&addr, num_workers, &handle).unwrap();
    let server = listener.incoming().for_each(|(sock, addr)| {
        let service = EndpointService::new(endpoint.clone(), &handle);
        proto.bind_connection(&handle, sock, addr, service);
        Ok(())
    });

    core.run(server).unwrap()
}

fn listener(addr: &SocketAddr, num_workers: usize, handle: &Handle) -> io::Result<TcpListener> {
    let listener = match *addr {
        SocketAddr::V4(_) => TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => TcpBuilder::new_v6()?,
    };
    configure_tcp(&listener, num_workers)?;
    listener.reuse_address(true)?;
    listener.bind(addr)?;
    let l = listener.listen(1024)?;
    TcpListener::from_listener(l, addr, handle)
}

#[cfg(not(windows))]
fn configure_tcp(tcp: &TcpBuilder, workers: usize) -> io::Result<()> {
    use net2::unix::UnixTcpBuilderExt;
    if workers > 1 {
        tcp.reuse_port(true)?;
    }
    Ok(())
}

#[cfg(windows)]
fn configure_tcp(_: &TcpBuilder, _: usize) -> io::Result<()> {
    Ok(())
}
