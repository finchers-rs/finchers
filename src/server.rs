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

use context::Context;
use endpoint::{Endpoint, EndpointError};
use response::Responder;
use task::Task;


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

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = RespondFuture<TaskFuture<E::Task>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let ctx = Context::from_hyper(req);
        let task = create_task_future(&self.endpoint, ctx);
        RespondFuture::new(task)
    }
}


pub(crate) fn create_task_future<E: Endpoint>(
    endpoint: &E,
    mut ctx: Context,
) -> Result<TaskFuture<E::Task>, EndpointError> {
    let mut result = endpoint.apply(&mut ctx);

    // check if the remaining path segments are exist.
    if ctx.count_remaining_segments() > 0 {
        result = Err(EndpointError::Skipped);
    }

    result.map(|task| TaskFuture::new(task, ctx))
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct TaskFuture<T: Task> {
    task: T,
    ctx: Context,
}

impl<T: Task> TaskFuture<T> {
    #[allow(missing_docs)]
    pub fn new(task: T, ctx: Context) -> Self {
        TaskFuture { task, ctx }
    }
}

impl<T: Task> Future for TaskFuture<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.task.poll(&mut self.ctx)
    }
}


/// The type of a future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct RespondFuture<F: Future>
where
    F::Item: Responder,
    F::Error: Responder,
{
    inner: Result<F, Option<EndpointError>>,
}

impl<F: Future> RespondFuture<F>
where
    F::Item: Responder,
    F::Error: Responder,
{
    #[allow(missing_docs)]
    pub fn new(result: Result<F, EndpointError>) -> Self {
        RespondFuture {
            inner: result.map_err(Some),
        }
    }
}

impl<F: Future> Future for RespondFuture<F>
where
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
