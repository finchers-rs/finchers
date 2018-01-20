use std::sync::Arc;
use futures::{Future, Stream};
use hyper;
use hyper::server::NewService;
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
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
    B: TcpBackend,
{
    new_service: S,
    http: Http,
    tcp: Tcp<B>,
}

impl<S, B> WorkerContext<S, B>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), ::hyper::Error> {
        for addr in &self.tcp.addrs {
            let incoming = self.tcp.backend.incoming(addr, &handle)?;
            let serve = self.http
                .inner()
                .serve_incoming(incoming, self.new_service.clone())
                .for_each(|conn| conn.map(|_| ()))
                .map_err(|_| ());
            handle.spawn(serve);
        }

        Ok(())
    }
}

pub fn start_multi_threaded<S, B>(application: Application<S, B>)
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
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
