use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

use futures::{future, Future, Poll, Stream};
use http::{Request, Response};
use hyper;
use hyper::server::{NewService, Service};
use num_cpus;
use tokio_core::reactor::{Core, Handle};

use response::ResponseBody;
use super::{const_service, ConstService, HttpService, NewHttpService};
use super::backend::TcpBackend;

#[derive(Debug)]
pub struct Server<S> {
    new_service: S,
    addrs: Option<Vec<SocketAddr>>,
    num_workers: usize,
    pipeline: bool,
    keep_alive: bool,
}

impl<S> Server<S>
where
    S: NewHttpService + Clone + Send + Sync + 'static,
    S::RequestBody: From<hyper::Body>,
{
    /// Create a new launcher from given service.
    pub fn new(new_service: S) -> Self {
        Server {
            new_service,
            addrs: None,
            num_workers: num_cpus::get(),
            pipeline: true,
            keep_alive: false,
        }
    }

    pub fn bind<T: ToSocketAddrs>(mut self, addrs: T) -> Self {
        self.addrs
            .get_or_insert_with(|| Default::default())
            .extend(addrs.to_socket_addrs().unwrap());
        self
    }

    /// Start the HTTP server with given configurations
    #[inline]
    pub fn run<B>(self, backend: B)
    where
        B: TcpBackend + Send + Sync + 'static,
    {
        let addrs = self.addrs
            .unwrap_or_else(|| vec!["0.0.0.0:4000".parse().unwrap()]);

        let ctx = Arc::new(WorkerContext {
            new_service: self.new_service,
            addrs,
            backend,
            pipeline: self.pipeline,
            keep_alive: self.keep_alive,
        });
        let spawn = || {
            let ctx = ctx.clone();
            thread::spawn(move || -> Result<(), hyper::Error> {
                let mut core = Core::new()?;
                let _ = ctx.spawn(&core.handle());
                core.run(future::empty())
            })
        };

        let mut handles = vec![];
        for _ in 0..self.num_workers {
            handles.push(spawn());
        }

        while !handles.is_empty() {
            let mut respawn = 0;
            for handle in handles.drain(..) {
                match handle.join() {
                    Ok(result) => result.unwrap(),
                    Err(_e) => respawn += 1,
                }
            }
            for _ in 0..respawn {
                handles.push(spawn());
            }
        }
    }
}

impl<S> Server<ConstService<S>>
where
    S: HttpService + Send + Sync + 'static,
    S::RequestBody: From<hyper::Body>,
{
    pub fn from_service(service: S) -> Self {
        Self::new(const_service(service))
    }
}

pub struct WorkerContext<S, B> {
    new_service: S,
    addrs: Vec<SocketAddr>,
    pipeline: bool,
    keep_alive: bool,
    backend: B,
}

impl<S, B> WorkerContext<S, B>
where
    S: NewHttpService + Clone + 'static,
    S::RequestBody: From<hyper::Body>,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), hyper::Error> {
        let mut http = hyper::server::Http::new();
        http.pipeline(self.pipeline);
        http.keep_alive(self.keep_alive);

        for addr in &self.addrs {
            let incoming = self.backend.incoming(addr, &handle)?;
            let new_service = CompatNewService {
                new_service: self.new_service.clone(),
            };
            let serve = http.serve_incoming(incoming, new_service)
                .for_each(|conn| conn.map(|_| ()))
                .map_err(|_| ());
            handle.spawn(serve);
        }

        Ok(())
    }
}

pub struct CompatNewService<S> {
    new_service: S,
}

impl<S> NewService for CompatNewService<S>
where
    S: NewHttpService,
    S::RequestBody: From<hyper::Body>,
{
    type Request = hyper::Request<hyper::Body>;
    type Response = hyper::Response<BodyWrapper<<S::ResponseBody as ResponseBody>::Stream>>;
    type Error = hyper::Error;
    type Instance = CompatService<S::Service>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        self.new_service
            .new_service()
            .map(|service| CompatService { service })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct CompatService<S> {
    service: S,
}

impl<S> Service for CompatService<S>
where
    S: HttpService,
    S::RequestBody: From<hyper::Body>,
{
    type Request = hyper::Request<hyper::Body>;
    type Response = hyper::Response<BodyWrapper<<S::ResponseBody as ResponseBody>::Stream>>;
    type Error = hyper::Error;
    type Future = CompatServiceFuture<S::Future>;

    #[inline]
    fn call(&self, req: Self::Request) -> Self::Future {
        CompatServiceFuture {
            future: self.service.call(Request::from(req).map(Into::into)),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct CompatServiceFuture<F> {
    future: F,
}

impl<F, Bd> Future for CompatServiceFuture<F>
where
    F: Future<Item = Response<Bd>, Error = io::Error>,
    Bd: ResponseBody,
{
    type Item = hyper::Response<BodyWrapper<Bd::Stream>>;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.future.poll());
        let item = hyper::Response::from(item.map(ResponseBody::into_stream).map(BodyWrapper));
        Ok(item.into())
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
