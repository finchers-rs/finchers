use std::borrow::Cow;
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

use endpoint::{Endpoint, EndpointError};
use response::IntoResponder;
use super::EndpointService;


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Server {
    addr: Cow<'static, str>,
    num_workers: usize,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            addr: "0.0.0.0:4000".into(),
            num_workers: 1,
        }
    }
}

#[allow(missing_docs)]
impl Server {
    pub fn bind<S: Into<Cow<'static, str>>>(mut self, addr: S) -> Self {
        self.addr = addr.into();
        self
    }

    pub fn num_workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }

    pub fn serve<E>(&self, endpoint: E)
    where
        E: Endpoint + Send + Sync + 'static,
        E::Item: IntoResponder,
        E::Error: IntoResponder + From<EndpointError>,
    {
        let endpoint = Arc::new(endpoint);
        let proto = Http::new();
        let addr = self.addr.parse().unwrap();
        let reuse_port = self.num_workers > 1;

        for _ in 0..(self.num_workers - 1) {
            let endpoint = endpoint.clone();
            let proto = proto.clone();
            thread::spawn(move || {
                let _ = Worker {
                    endpoint,
                    proto,
                    addr: &addr,
                    reuse_port,
                }.run();
            });
        }
        let _ = Worker {
            endpoint,
            proto,
            addr: &addr,
            reuse_port,
        }.run();
    }
}


#[derive(Debug)]
struct Worker<'a, E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    endpoint: E,
    proto: Http<Chunk>,
    addr: &'a SocketAddr,
    reuse_port: bool,
}

impl<'a, E> Worker<'a, E>
where
    E: Endpoint + Clone + 'static,
    E::Item: IntoResponder,
    E::Error: IntoResponder + From<EndpointError>,
{
    fn run(&self) -> io::Result<()> {
        let mut core = Core::new()?;
        let handle = core.handle();

        let server = self.build_listener(&handle)?
            .incoming()
            .for_each(|(sock, addr)| {
                let service = EndpointService::new(self.endpoint.clone());
                self.proto.bind_connection(&handle, sock, addr, service);
                Ok(())
            });

        core.run(server)
    }

    fn build_listener(&self, handle: &Handle) -> io::Result<TcpListener> {
        let listener = match *self.addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4()?,
            SocketAddr::V6(..) => TcpBuilder::new_v6()?,
        };
        configure_tcp(&listener, self.reuse_port)?;
        listener.reuse_address(true)?;
        listener.bind(&self.addr)?;
        let l = listener.listen(1024)?;
        TcpListener::from_listener(l, &self.addr, handle)
    }
}

#[cfg(not(windows))]
fn configure_tcp(tcp: &TcpBuilder, reuse_port: bool) -> io::Result<()> {
    use net2::unix::UnixTcpBuilderExt;
    if reuse_port {
        tcp.reuse_port(true)?;
    }
    Ok(())
}

#[cfg(windows)]
fn configure_tcp(_: &TcpBuilder, _: bool) -> io::Result<()> {
    Ok(())
}
