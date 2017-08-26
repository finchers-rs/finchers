//! Definition of HTTP services for Hyper

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::io;

use futures::{Future, IntoFuture, Stream};
use futures::future::{AndThen, Flatten, FutureResult, MapErr, Then};
use hyper;
use hyper::server::Http;
use net2::TcpBuilder;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_service::Service;

use context::{self, Context};
use endpoint::{Endpoint, NewEndpoint};
use errors::*;
use request;
use response::Responder;


/// A wrapper for `Endpoint`s, to provide HTTP services
#[derive(Debug, Clone)]
pub struct EndpointService<E> {
    endpoint: E,
    handle: Handle,
}

impl<E> Service for EndpointService<E>
where
    E: NewEndpoint,
    E::Item: Responder,
    E::Error: Into<FinchersError>,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Then<
        AndThen<
            Flatten<FutureResult<MapErr<E::Future, fn(E::Error) -> FinchersError>, FinchersError>>,
            FinchersResult<hyper::Response>,
            fn(E::Item) -> FinchersResult<hyper::Response>,
        >,
        Result<hyper::Response, hyper::Error>,
        fn(FinchersResult<hyper::Response>)
            -> Result<hyper::Response, hyper::Error>,
    >;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let (req, body) = request::reconstruct(req);
        let base = context::RequestInfo::new(&req, body);
        let mut ctx = Context::from(&base);

        let endpoint = self.endpoint.new_endpoint(&self.handle);
        let mut result = endpoint
            .apply(&mut ctx)
            .map_err(|_| FinchersErrorKind::NotFound.into());
        if ctx.next_segment().is_some() {
            result = Err(FinchersErrorKind::NotFound.into());
        }

        result
            .map(|fut| {
                fut.map_err(Into::into as fn(E::Error) -> FinchersError)
            })
            .into_future()
            .flatten()
            .and_then(
                (|res| {
                    res.respond()
                        .map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
                }) as fn(E::Item) -> FinchersResult<hyper::Response>,
            )
            .then(|response| {
                Ok(response.unwrap_or_else(|err| err.into_response()))
            })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Server<E> {
    endpoint: E,
    addr: Option<&'static str>,
    num_workers: Option<usize>,
}

impl<E: NewEndpoint> Server<E> {
    #[allow(missing_docs)]
    pub fn new(endpoint: E) -> Self {
        Self {
            endpoint,
            addr: None,
            num_workers: None,
        }
    }

    #[allow(missing_docs)]
    pub fn bind(mut self, addr: &'static str) -> Self {
        self.addr = Some(addr);
        self
    }

    #[allow(missing_docs)]
    pub fn num_workers(mut self, n: usize) -> Self {
        self.num_workers = Some(n);
        self
    }
}

impl<E> Server<E>
where
    E: NewEndpoint + Send + Sync + 'static,
    E::Item: Responder,
    E::Error: Into<FinchersError>,
{
    /// Start the HTTP server, with given endpoint and listener address.
    pub fn run_http(self) {
        let endpoint = Arc::new(self.endpoint);
        let addr = self.addr.unwrap_or("0.0.0.0:4000").parse().unwrap();
        let num_workers = self.num_workers.unwrap_or(1);

        for _ in 0..(num_workers - 1) {
            let endpoint = endpoint.clone();
            thread::spawn(move || { serve(endpoint, num_workers, &addr); });
        }
        serve(endpoint.clone(), num_workers, &addr);
    }
}

fn serve<E>(endpoint: E, num_workers: usize, addr: &SocketAddr)
where
    E: NewEndpoint + Clone + 'static,
    E::Item: Responder,
    E::Error: Into<FinchersError>,
{
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let proto = Http::new();

    let listener = listener(&addr, num_workers, &handle).unwrap();
    let server = listener.incoming().for_each(|(sock, addr)| {
        proto.bind_connection(
            &handle,
            sock,
            addr,
            EndpointService {
                endpoint: endpoint.clone(),
                handle: handle.clone(),
            },
        );
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
