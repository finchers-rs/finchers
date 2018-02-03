use std::sync::Arc;
use std::thread;
use futures::{future, Future, Stream};
use hyper;
use hyper::server::{self, NewService};
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

pub struct WorkerContext<S, B> {
    new_service: S,
    http: Http,
    tcp: Tcp<B>,
}

impl<S, Bd, B> WorkerContext<S, B>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response<Bd>, Error = hyper::Error> + Clone + 'static,
    Bd: Stream<Error = hyper::Error> + 'static,
    Bd::Item: AsRef<[u8]> + 'static,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), hyper::Error> {
        let mut http = server::Http::new();
        http.pipeline(self.http.pipeline);
        http.keep_alive(self.http.keep_alive);

        for addr in &self.tcp.addrs {
            let incoming = self.tcp.backend.incoming(addr, &handle)?;
            let serve = http.serve_incoming(incoming, self.new_service.clone())
                .for_each(|conn| conn.map(|_| ()))
                .map_err(|_| ());
            handle.spawn(serve);
        }

        Ok(())
    }
}

pub fn start_multi_threaded<S, Bd, B>(application: Application<S, B>) -> Result<(), hyper::Error>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response<Bd>, Error = hyper::Error> + Clone + 'static,
    Bd: Stream<Error = hyper::Error> + 'static,
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
