use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use std::thread;
use futures::{future, Future, Poll, Stream};
use http::{Request, Response};
use hyper;
use tokio_core::reactor::{Core, Handle};
use tokio_service::{NewService, Service};

use request::body::BodyStream;
use super::{Application, Http, Tcp, TcpBackend};

/// Worker level configuration
#[derive(Debug)]
pub struct Worker {
    /// The number of worker threads
    pub num_workers: usize,
}

impl Default for Worker {
    fn default() -> Self {
        Worker { num_workers: 1 }
    }
}

pub struct WorkerContext<S, B> {
    new_service: S,
    http: Http,
    tcp: Tcp<B>,
}

impl<S, Bd, B> WorkerContext<S, B>
where
    S: NewService<Request = Request<BodyStream>, Response = Response<Bd>, Error = io::Error> + Clone + 'static,
    Bd: Stream<Error = io::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), hyper::Error> {
        let mut http = hyper::server::Http::new();
        http.pipeline(self.http.pipeline);
        http.keep_alive(self.http.keep_alive);

        for addr in &self.tcp.addrs {
            let incoming = self.tcp.backend.incoming(addr, &handle)?;
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

pub fn start_multi_threaded<S, Bd, B>(application: Application<S, B>) -> Result<(), hyper::Error>
where
    S: NewService<Request = Request<BodyStream>, Response = Response<Bd>, Error = io::Error> + Clone + 'static,
    Bd: Stream<Error = io::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
    B: TcpBackend,
    // additional trait bounds in the case of multi-threaded condition
    S: Send + Sync,
    B: Send + Sync + 'static,
{
    let (new_service, http, mut tcp, worker) = application.deconstruct();
    tcp.normalize_addrs();

    let ctx = Arc::new(WorkerContext {
        new_service,
        http,
        tcp,
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
    for _ in 0..worker.num_workers {
        handles.push(spawn());
    }

    while !handles.is_empty() {
        let mut respawn = 0;
        for handle in handles.drain(..) {
            match handle.join() {
                Ok(result) => result?,
                Err(_e) => respawn += 1,
            }
        }
        for _ in 0..respawn {
            handles.push(spawn());
        }
    }

    Ok(())
}

pub struct CompatNewService<S> {
    new_service: S,
}

impl<S, Bd> NewService for CompatNewService<S>
where
    S: NewService<Request = Request<BodyStream>, Response = Response<Bd>, Error = io::Error>,
    Bd: Stream<Error = io::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
{
    type Request = hyper::Request<hyper::Body>;
    type Response = hyper::Response<BodyWrapper<Bd>>;
    type Error = hyper::Error;
    type Instance = CompatService<S::Instance>;

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

impl<S, Bd> Service for CompatService<S>
where
    S: Service<Request = Request<BodyStream>, Response = Response<Bd>, Error = io::Error>,
    Bd: Stream<Error = io::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
{
    type Request = hyper::Request<hyper::Body>;
    type Response = hyper::Response<BodyWrapper<Bd>>;
    type Error = hyper::Error;
    type Future = CompatServiceFuture<S::Future, Bd>;

    #[inline]
    fn call(&self, req: Self::Request) -> Self::Future {
        CompatServiceFuture {
            future: self.service.call(Request::from(req).map(Into::into)),
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct CompatServiceFuture<F, Bd> {
    future: F,
    _marker: PhantomData<fn() -> Bd>,
}

impl<F, Bd> Future for CompatServiceFuture<F, Bd>
where
    F: Future<Item = Response<Bd>, Error = io::Error>,
    Bd: Stream<Error = io::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
{
    type Item = hyper::Response<BodyWrapper<Bd>>;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.future.poll());
        let item = hyper::Response::from(item.map(BodyWrapper));
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
