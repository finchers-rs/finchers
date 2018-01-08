//! A lancher of the HTTP services

pub mod backend;

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use futures::{Future, Stream};
use hyper::Chunk;
use tokio_core::reactor::{Core, Handle};

use endpoint::Endpoint;
use responder::IntoResponder;
use service::{EndpointServiceFactory, ServiceFactory};

pub use self::backend::TcpBackend;

/// HTTP-level configuration
#[derive(Debug)]
pub struct Http(::hyper::server::Http<Chunk>);

impl Default for Http {
    fn default() -> Self {
        Http(::hyper::server::Http::new())
    }
}

impl Http {
    /// Enable or disable `Keep-alive` option
    pub fn keep_alive(&mut self, enabled: bool) -> &mut Self {
        self.0.keep_alive(enabled);
        self
    }

    /// Enable pipeline mode
    pub fn pipeline(&mut self, enabled: bool) -> &mut Self {
        self.0.pipeline(enabled);
        self
    }
}

/// TCP level configuration
#[derive(Debug)]
pub struct Tcp<B = backend::DefaultBackend> {
    addrs: Vec<SocketAddr>,
    backend: B,
}

impl Default for Tcp<backend::DefaultBackend> {
    fn default() -> Self {
        Tcp {
            addrs: vec![],
            backend: Default::default(),
        }
    }
}

impl<B> Tcp<B> {
    /// Create a new instance of `Tcp` with given backend
    pub fn new(backend: B) -> Self {
        Tcp {
            backend,
            addrs: vec![],
        }
    }

    /// Set the listener addresses.
    pub fn set_addrs<S>(&mut self, addrs: S) -> io::Result<()>
    where
        S: ToSocketAddrs,
    {
        self.addrs = addrs.to_socket_addrs()?.collect();
        Ok(())
    }

    /// Returns the mutable reference of the inner backend
    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }
}

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

/// The launcher of HTTP application.
#[derive(Debug)]
pub struct Application<S, B>
where
    S: ServiceFactory,
    B: TcpBackend,
{
    /// The instance of `ServiceFactory`
    service: S,

    /// HTTP-level configuration
    proto: Http,

    /// TCP-level configuration
    tcp: Tcp<B>,

    /// The worker's configuration
    worker: Worker,
}

impl<S, B> Application<S, B>
where
    S: ServiceFactory,
    B: TcpBackend,
{
    /// Create a new launcher from given service and TCP backend.
    pub fn new(service: S, backend: B) -> Self {
        Application {
            service,
            proto: Http::default(),
            worker: Worker::default(),
            tcp: Tcp {
                addrs: vec![],
                backend,
            },
        }
    }

    /// Returns a mutable reference of the service.
    ///
    /// ```ignore
    /// application.service().set_secret_key(b"xxxx");
    /// ```
    pub fn service(&mut self) -> &mut S {
        &mut self.service
    }

    /// Returns a mutable reference of the HTTP configuration
    ///
    /// ```ignore
    /// application.http().keep_alive(true);
    /// application.http().pipeline(true);
    /// ```
    pub fn http(&mut self) -> &mut Http {
        &mut self.proto
    }

    /// Returns a mutable reference of the TCP configuration
    ///
    /// ```ignore
    /// application.tcp().append_addr("0.0.0.0:4000");
    /// application.tcp().reuse_port(true);
    /// application.tcp().backend().set_identity_path("identity.p12");
    /// ```
    pub fn tcp(&mut self) -> &mut Tcp<B> {
        &mut self.tcp
    }

    /// Returns a mutable reference of the worker configuration
    ///
    /// ```ignore
    /// application.worker().num_workers = 1;
    pub fn worker(&mut self) -> &mut Worker {
        &mut self.worker
    }
}

impl<S: ServiceFactory> Application<S, backend::DefaultBackend> {
    /// Create a new instance of Application from given service
    pub fn from_service(service: S) -> Self {
        Self::new(service, Default::default())
    }
}

impl<E> Application<EndpointServiceFactory<E>, backend::DefaultBackend>
where
    E: Endpoint,
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
    /// Create a lancher from given `Endpoint`.
    pub fn from_endpoint(endpoint: E) -> Self {
        Self::from_service(EndpointServiceFactory::new(endpoint))
    }
}

impl<S, B> Application<S, B>
where
    S: ServiceFactory + Send + Sync + 'static,
    B: TcpBackend + Send + Sync + 'static,
{
    /// Start the HTTP server with given configurations
    pub fn run(mut self) {
        if self.tcp.addrs.is_empty() {
            println!("[info] Use default listener addresses.");
            self.tcp.addrs.push("0.0.0.0:4000".parse().unwrap());
            self.tcp.addrs.push("[::0]:4000".parse().unwrap());
        } else {
            let set: ::std::collections::HashSet<_> = self.tcp.addrs.into_iter().collect();
            self.tcp.addrs = set.into_iter().collect();
        }

        let ctx = Arc::new(WorkerContext {
            service: Arc::new(self.service),
            http: self.proto,
            tcp: self.tcp,
        });

        let mut handles = vec![];
        for _ in 0..self.worker.num_workers {
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
}

struct WorkerContext<S, B>
where
    S: ServiceFactory + 'static,
    B: TcpBackend,
{
    service: Arc<S>,
    http: Http,
    tcp: Tcp<B>,
}

impl<S, B> WorkerContext<S, B>
where
    S: ServiceFactory + 'static,
    B: TcpBackend,
{
    fn spawn(&self, handle: &Handle) -> Result<(), ::hyper::Error> {
        let new_service = NewService {
            inner: self.service.clone(),
            handle: handle.clone(),
        };

        for addr in &self.tcp.addrs {
            let incoming = self.tcp.backend.incoming(addr, &handle)?;
            let serve = self.http
                .0
                .serve_incoming(incoming, new_service.clone())
                .for_each(|conn| conn.map(|_| ()))
                .map_err(|_| ());
            handle.spawn(serve);
        }

        Ok(())
    }
}

struct NewService<S: ServiceFactory> {
    inner: Arc<S>,
    handle: Handle,
}

impl<S: ServiceFactory> Clone for NewService<S> {
    fn clone(&self) -> Self {
        NewService {
            inner: self.inner.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<S: ServiceFactory> ::hyper::server::NewService for NewService<S> {
    type Request = ::hyper::Request;
    type Response = ::hyper::Response;
    type Error = ::hyper::Error;
    type Instance = S::Service;

    fn new_service(&self) -> io::Result<Self::Instance> {
        self.inner.new_service(&self.handle)
    }
}
