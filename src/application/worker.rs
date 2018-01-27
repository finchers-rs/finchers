use std::io;
use std::sync::Arc;
use futures::{Future, Poll, Stream};
use hyper::{self, Body, Error};
use hyper::server::{NewService, Service};
use http_crate::{Request, Response};
use tokio_core::reactor::{Core, Handle};

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

pub struct WorkerContext<S, B>
where
    S: NewService<Request = Request<Body>, Response = Response<Body>, Error = Error> + Clone + 'static,
    B: TcpBackend,
{
    new_service: S,
    http: Http,
    tcp: Tcp<B>,
}

impl<S, B> WorkerContext<S, B>
where
    S: NewService<Request = Request<Body>, Response = Response<Body>, Error = Error> + Clone + 'static,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), ::hyper::Error> {
        for addr in &self.tcp.addrs {
            let incoming = self.tcp.backend.incoming(addr, &handle)?;
            let new_service = NewCompatService {
                new_service: self.new_service.clone(),
            };
            let serve = self.http
                .inner()
                .serve_incoming(incoming, new_service)
                .for_each(|conn| conn.map(|_| ()))
                .map_err(|_| ());
            handle.spawn(serve);
        }

        Ok(())
    }
}

pub fn start_multi_threaded<S, B>(application: Application<S, B>)
where
    S: NewService<Request = Request<Body>, Response = Response<Body>, Error = Error> + Clone + 'static,
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

    let mut handles = vec![];
    for _ in 0..worker.num_workers {
        let ctx = ctx.clone();
        handles.push(::std::thread::spawn(
            move || -> Result<(), ::hyper::Error> {
                let mut core = Core::new()?;
                let _ = ctx.spawn(&core.handle());
                core.run(::futures::future::empty())
            },
        ));
    }

    for handle in handles {
        let _ = handle.join();
    }
}

struct NewCompatService<S> {
    new_service: S,
}

impl<S> NewService for NewCompatService<S>
where
    S: NewService<Request = Request<Body>, Response = Response<Body>, Error = Error>,
{
    type Request = hyper::Request<Body>;
    type Response = hyper::Response<Body>;
    type Error = hyper::Error;
    type Instance = CompatService<S::Instance>;

    #[inline]
    fn new_service(&self) -> io::Result<Self::Instance> {
        self.new_service
            .new_service()
            .map(|service| CompatService { service })
    }
}

struct CompatService<S> {
    service: S,
}

impl<S> Service for CompatService<S>
where
    S: Service<Request = Request<Body>, Response = Response<Body>, Error = Error>,
{
    type Request = hyper::Request<Body>;
    type Response = hyper::Response<Body>;
    type Error = hyper::Error;
    type Future = CompatFuture<S::Future>;

    #[inline]
    fn call(&self, request: Self::Request) -> Self::Future {
        CompatFuture {
            future: self.service.call(request.into()),
        }
    }
}

struct CompatFuture<F> {
    future: F,
}

impl<F> Future for CompatFuture<F>
where
    F: Future<Item = Response<Body>, Error = Error>,
{
    type Item = hyper::Response<Body>;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map(|t| t.map(Into::into))
    }
}
