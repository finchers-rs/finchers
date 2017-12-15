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
use endpoint::{Endpoint, EndpointError};
use request;
use response::Responder;
use test;


/// A wrapper of a `NewEndpoint`, to provide hyper's HTTP services
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    endpoint: E,
    handle: Handle,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    fn apply_endpoint(&self, ctx: &mut Context) -> Result<E::Task, EndpointError> {
        // Create a new endpoint from the inner factory. and evaluate it.
        let mut result = self.endpoint.apply(ctx);

        // check if the remaining path segments are exist.
        if ctx.next_segment().is_some() {
            result = Err(EndpointError::Skipped);
        }

        result
    }
}

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = EndpointFuture<test::EndpointFuture<E>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        // reconstruct the instance of `hyper::Request` and parse its path and queries.
        let (req, body) = request::reconstruct(req);
        let info = RequestInfo::new(req, body);

        // Create the instance of `Context` from the reference of `RequestInfo`.
        let mut ctx = Context::new(info);

        // create and apply the endpoint to parsed `RequestInfo`
        let result = self.apply_endpoint(&mut ctx);

        EndpointFuture {
            inner: match result {
                Ok(task) => Ok(test::EndpointFuture { task, ctx }),
                Err(err) => Err(Some(err)),
            },
        }
    }
}


/// The type of a future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct EndpointFuture<F>
where
    F: Future,
    F::Item: Responder,
    F::Error: Responder,
{
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
        let addr = self.addr.unwrap_or("0.0.0.0:4000".into()).parse().unwrap();
        let num_workers = self.num_workers.unwrap_or(1);

        for _ in 0..(num_workers - 1) {
            let endpoint = endpoint.clone();
            thread::spawn(move || {
                serve(endpoint, num_workers, &addr);
            });
        }
        serve(endpoint.clone(), num_workers, &addr);
    }
}

fn serve<E>(endpoint: Arc<E>, num_workers: usize, addr: &SocketAddr)
where
    E: Endpoint + 'static,
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
