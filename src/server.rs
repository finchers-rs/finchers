//! Definition of HTTP services for Hyper

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use futures::{Future, Poll, Stream};
use hyper;
use hyper::server::Http;
use net2::TcpBuilder;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_service::Service;

use context::{Context, RequestInfo};
use endpoint::{Endpoint, NewEndpoint};
use endpoint::result::{EndpointError, EndpointResult};
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
    E::Error: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointFuture<E::Future>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        // reconstruct the instance of `hyper::Request` and parse its path and queries.
        let (req, body) = request::reconstruct(req);
        let info = RequestInfo::new(&req, body);

        // create and apply the endpoint to parsed `RequestInfo`
        let inner = self.apply_endpoint(&info);
        EndpointFuture {
            inner: inner.map_err(Some),
        }
    }
}

impl<E> EndpointService<E>
where
    E: NewEndpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    fn apply_endpoint(&self, req: &RequestInfo) -> EndpointResult<E::Future> {
        // Create the instance of `Context` from the reference of `RequestInfo`.
        let mut ctx = Context::from(req);

        // Create a new endpoint from the inner factory. and evaluate it.
        let endpoint = self.endpoint.new_endpoint(&self.handle);
        let mut result = endpoint.apply(&mut ctx);

        // check if the remaining path segments are exist.
        if ctx.next_segment().is_some() {
            result = Err(EndpointError::Skipped);
        }

        result
    }
}


#[doc(hidden)]
#[derive(Debug)]
pub struct EndpointFuture<F> {
    inner: Result<F, Option<EndpointError>>,
}

impl<F> Future for EndpointFuture<F>
where
    F: Future,
    F::Item: Responder,
    F::Error: Responder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check the result of `Endpoint::apply()`.
        let inner = match self.inner.as_mut() {
            Ok(inner) => inner,
            Err(err) => {
                let err = err.take().expect("cannot reject twice");
                return Ok(err.into_response().into());
            }
        };

        // Query the future returned from the endpoint
        let item = inner.poll();
        // ...and convert its success/error value to `hyper::Response`.
        let item = item.map(|item| item.map(Responder::into_response))
            .map_err(Responder::into_response);

        Ok(item.unwrap_or_else(Into::into))
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
    E::Error: Responder,
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
    E::Error: Responder,
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
