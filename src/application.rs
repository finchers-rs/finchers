#![allow(missing_docs)]
#![allow(missing_debug_implementations)]
#![warn(warnings)]

use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use hyper::Chunk;
use tokio_core::reactor::{Core, Handle};
use service::ServiceFactory;
use endpoint::Endpoint;
use service::EndpointServiceFactory;
use responder::IntoResponder;
use futures::{Future, Stream};

pub mod backend {
    use std::io;
    use native_tls::TlsAcceptor;
    use native_tls::Pkcs12;
    use futures::Stream;
    use tokio_io::{AsyncRead, AsyncWrite};
    use std::net::SocketAddr;
    use tokio_core::reactor::Handle;
    use tokio_tls::TlsAcceptorExt;
    use futures::Future;

    pub trait TcpBackend {
        type Io: AsyncRead + AsyncWrite + 'static;
        type Incoming: Stream<Item = Self::Io, Error = io::Error> + 'static;

        fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming>;
    }

    use net2::TcpBuilder;
    #[cfg(unix)]
    use net2::unix::UnixTcpBuilderExt;

    fn listener(addr: &SocketAddr, handle: &Handle) -> io::Result<::tokio_core::net::TcpListener> {
        let listener = match *addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4()?,
            SocketAddr::V6(..) => TcpBuilder::new_v6()?,
        };

        listener.reuse_address(true)?;
        #[cfg(unix)]
        listener.reuse_port(true)?;

        listener.bind(addr)?;
        let l = listener.listen(128)?;

        ::tokio_core::net::TcpListener::from_listener(l, addr, handle)
    }

    #[derive(Default, Debug)]
    pub struct DefaultBackend {}

    impl TcpBackend for DefaultBackend {
        type Io = ::tokio_core::net::TcpStream;
        type Incoming = ::futures::stream::Map<
            ::tokio_core::net::Incoming,
            fn((::tokio_core::net::TcpStream, SocketAddr)) -> ::tokio_core::net::TcpStream,
        >;
        fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming> {
            Ok(listener(addr, handle)?.incoming().map(
                (|(sock, _)| sock) as fn((::tokio_core::net::TcpStream, SocketAddr)) -> ::tokio_core::net::TcpStream,
            ))
        }
    }

    pub struct TlsBackend {
        acceptor: TlsAcceptor,
    }

    impl TlsBackend {
        pub fn from_pkcs12(pkcs12: Pkcs12) -> Result<Self, ::native_tls::Error> {
            let acceptor = TlsAcceptor::builder(pkcs12)?.build()?;
            Ok(TlsBackend { acceptor })
        }
    }

    impl From<TlsAcceptor> for TlsBackend {
        fn from(acceptor: TlsAcceptor) -> TlsBackend {
            TlsBackend { acceptor }
        }
    }

    impl TcpBackend for TlsBackend {
        type Io = ::tokio_tls::TlsStream<::tokio_core::net::TcpStream>;
        type Incoming = Box<Stream<Item = Self::Io, Error = io::Error>>;
        fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming> {
            let acceptor = self.acceptor.clone();
            Ok(Box::new(listener(addr, handle)?.incoming().and_then(
                move |(sock, _)| {
                    acceptor
                        .accept_async(sock)
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Accept error: {}", err)))
                },
            )))
        }
    }
}
pub use self::backend::TcpBackend;

#[derive(Debug)]
pub struct Http(::hyper::server::Http<Chunk>);

impl Default for Http {
    fn default() -> Self {
        Http(::hyper::server::Http::new())
    }
}
impl Http {
    pub fn keep_alive(&mut self, enabled: bool) -> &mut Self {
        self.0.keep_alive(enabled);
        self
    }

    pub fn pipeline(&mut self, enabled: bool) -> &mut Self {
        self.0.pipeline(enabled);
        self
    }
}

///
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
    pub fn new(backend: B) -> Self {
        Tcp {
            backend,
            addrs: vec![],
        }
    }

    pub fn set_addrs<S>(&mut self, addrs: S) -> io::Result<()>
    where
        S: ToSocketAddrs,
    {
        self.addrs = addrs.to_socket_addrs()?.collect();
        Ok(())
    }

    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }
}

#[derive(Debug, Default)]
pub struct Worker {
    /// The number of worker threads
    pub num_workers: usize,
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
    ///
    /// ```ignore
    /// let mut application = Application::from_endpoint(endpoint);
    /// application.service().set_secret_key("xxx");
    /// application.run();
    /// ```
    pub fn from_endpoint(endpoint: E) -> Self {
        Self::from_service(EndpointServiceFactory::new(endpoint))
    }
}

impl<S, B> Application<S, B>
where
    S: ServiceFactory + Send + Sync + 'static,
    B: TcpBackend + Send + Sync + 'static,
{
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
                .for_each(|_| Ok(()))
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
