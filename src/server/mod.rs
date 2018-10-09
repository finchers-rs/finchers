//! The implementation of HTTP server based on hyper and tower-service.

pub mod middleware;

pub use self::imp::{start, ServerConfig, ServerError, ServerResult, ServiceBuilder};

mod imp {
    use std::error;
    use std::fmt;
    use std::net::{IpAddr, SocketAddr};
    use std::sync::Arc;

    use failure;
    use failure::Fallible;
    use futures::{future, Future, Stream};
    use http::{Request, Response};
    use hyper::body::{Body, Payload};
    use hyper::server as hyper_server;
    use hyper::server::conn::AddrIncoming;
    use hyper::service as hyper_service;
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::runtime::Runtime;
    use tower_service;
    use tower_service::NewService;
    #[cfg(feature = "tower-web")]
    use tower_web;

    use app::App;
    use endpoint::OutputEndpoint;
    use rt::{with_set_runtime_mode, RuntimeMode};

    #[cfg(feature = "tower-web")]
    use super::middleware::TowerWebMiddleware;
    use super::middleware::{Chain, Middleware};

    /// The error type which will be returned from `ServiceBuilder::serve()`.
    #[derive(Debug)]
    pub struct ServerError {
        inner: failure::Error,
    }

    impl fmt::Display for ServerError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "failed to start the server: {}", self.inner)
        }
    }

    impl error::Error for ServerError {
        fn description(&self) -> &str {
            "failed to start the server"
        }
    }

    /// A type alias of `Result<T, E>` whose error type is restrected to `ServerError`.
    pub type ServerResult<T> = Result<T, ServerError>;

    /// Create an instance of `ServiceBuilder` from the specified endpoint.
    pub fn start<E>(endpoint: E) -> ServiceBuilder<App<E>>
    where
        for<'a> E: OutputEndpoint<'a> + 'static,
    {
        ServiceBuilder::new(App::new(endpoint))
    }

    /// A builder of HTTP service.
    #[derive(Debug)]
    pub struct ServiceBuilder<S> {
        new_service: S,
    }

    impl<S> ServiceBuilder<S>
    where
        S: NewService,
    {
        /// Creates a new `ServerBuilder` from the specified NewService.
        pub fn new(new_service: S) -> Self {
            ServiceBuilder { new_service }
        }

        /// Wraps the inner service into the specified middleware.
        pub fn with_middleware<M>(self, middleware: M) -> ServiceBuilder<Chain<S, M>>
        where
            M: Middleware<S::Service> + Clone,
        {
            ServiceBuilder {
                new_service: Chain::new(self.new_service, middleware),
            }
        }

        /// Wraps the inner service into the specified Tower-web middleware.
        #[cfg(feature = "tower-web")]
        pub fn with_tower_middleware<M>(
            self,
            middleware: M,
        ) -> ServiceBuilder<Chain<S, TowerWebMiddleware<M>>>
        where
            M: tower_web::middleware::Middleware<S::Service>,
        {
            ServiceBuilder {
                new_service: Chain::new(self.new_service, TowerWebMiddleware::new(middleware)),
            }
        }
    }

    impl<S> NewService for ServiceBuilder<S>
    where
        S: NewService,
    {
        type Request = S::Request;
        type Response = S::Response;
        type Error = S::Error;
        type Service = S::Service;
        type InitError = S::InitError;
        type Future = S::Future;

        #[inline]
        fn new_service(&self) -> Self::Future {
            self.new_service.new_service()
        }
    }

    impl<S, Bd> ServiceBuilder<S>
    where
        S: NewService<Request = Request<Body>, Response = Response<Bd>> + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        S::Service: Send,
        S::Future: Send + 'static,
        <S::Service as tower_service::Service>::Future: Send + 'static,
        Bd: Payload,
    {
        /// Start the server with the specified configuration.
        pub fn serve(self, config: impl ServerConfig) -> ServerResult<()> {
            self.serve_with_graceful_shutdown(config, future::empty::<(), ()>())
        }

        /// Start the server with the specified configuration and shutdown signal.
        pub fn serve_with_graceful_shutdown(
            self,
            server_config: impl ServerConfig,
            signal: impl Future<Item = ()> + Send + 'static,
        ) -> ServerResult<()> {
            server_config
                .build()
                .map_err(|inner| ServerError { inner })?
                .serve_with_graceful_shutdown(self, signal)
                .map_err(|inner| ServerError { inner })
        }
    }

    #[derive(Debug)]
    pub struct Server<I> {
        builder: hyper_server::Builder<I>,
        rt: Runtime,
    }

    impl<I> Server<I>
    where
        I: Stream + Send + 'static,
        I::Item: AsyncRead + AsyncWrite + Send + 'static,
        I::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    {
        fn serve_with_graceful_shutdown<S, Bd>(
            self,
            new_service: S,
            signal: impl Future<Item = ()> + Send + 'static,
        ) -> Fallible<()>
        where
            S: NewService<Request = Request<Body>, Response = Response<Bd>> + Send + Sync + 'static,
            S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
            S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
            S::Service: Send,
            S::Future: Send + 'static,
            <S::Service as tower_service::Service>::Future: Send + 'static,
            Bd: Payload,
        {
            let Self { builder, mut rt } = self;

            // Put the instance of new_service into the heap and ensure that
            // it lives until enter the scope.
            //
            // It implies that all tasks spawned by Tokio runtime must be dropped
            // after executing `shutdown_on_idle()`.
            let new_service = Arc::new(new_service);

            let mut server = builder
                .serve(NewHttpService(new_service.clone()))
                .with_graceful_shutdown(signal)
                .map_err(|err| error!("server error: {}", err));

            let server = future::poll_fn(move || {
                with_set_runtime_mode(RuntimeMode::ThreadPool, || server.poll())
            });

            rt.spawn(server);
            rt.shutdown_on_idle().wait().unwrap();
            Ok(())
        }
    }

    /// A trait representing the configuration to start the HTTP server.
    pub trait ServerConfig: ServerConfigImpl {}

    impl<'a> ServerConfig for &'a str {}
    impl ServerConfig for String {}
    impl<I: Into<IpAddr>> ServerConfig for (I, u16) {}
    impl ServerConfig for SocketAddr {}
    impl<'a> ServerConfig for &'a SocketAddr {}

    pub trait ServerConfigImpl {
        type Item: AsyncRead + AsyncWrite + Send + 'static;
        type Error: Into<Box<dyn error::Error + Send + Sync + 'static>>;
        type Incoming: Stream<Item = Self::Item, Error = Self::Error> + Send + 'static;

        fn build(self) -> Fallible<Server<Self::Incoming>>;
    }

    impl<'a> ServerConfigImpl for &'a str {
        type Item = <AddrIncoming as Stream>::Item;
        type Error = <AddrIncoming as Stream>::Error;
        type Incoming = AddrIncoming;

        fn build(self) -> Fallible<Server<Self::Incoming>> {
            ServerConfigImpl::build(&self.parse::<SocketAddr>()?)
        }
    }

    impl ServerConfigImpl for String {
        type Item = <AddrIncoming as Stream>::Item;
        type Error = <AddrIncoming as Stream>::Error;
        type Incoming = AddrIncoming;

        fn build(self) -> Fallible<Server<Self::Incoming>> {
            ServerConfigImpl::build(self.as_str())
        }
    }

    impl<I: Into<IpAddr>> ServerConfigImpl for (I, u16) {
        type Item = <AddrIncoming as Stream>::Item;
        type Error = <AddrIncoming as Stream>::Error;
        type Incoming = AddrIncoming;

        fn build(self) -> Fallible<Server<Self::Incoming>> {
            ServerConfigImpl::build(SocketAddr::from(self))
        }
    }

    impl ServerConfigImpl for SocketAddr {
        type Item = <AddrIncoming as Stream>::Item;
        type Error = <AddrIncoming as Stream>::Error;
        type Incoming = AddrIncoming;

        fn build(self) -> Fallible<Server<Self::Incoming>> {
            ServerConfigImpl::build(&self)
        }
    }

    impl<'a> ServerConfigImpl for &'a SocketAddr {
        type Item = <AddrIncoming as Stream>::Item;
        type Error = <AddrIncoming as Stream>::Error;
        type Incoming = AddrIncoming;

        fn build(self) -> Fallible<Server<Self::Incoming>> {
            let builder = hyper_server::Server::try_bind(self)?;
            let rt = Runtime::new()?;
            Ok(Server { builder, rt })
        }
    }

    #[derive(Debug)]
    struct NewHttpService<S>(S);

    impl<S, ReqBody, ResBody> hyper_service::NewService for NewHttpService<S>
    where
        S: tower_service::NewService<Request = Request<ReqBody>, Response = Response<ResBody>>,
        ReqBody: Payload,
        ResBody: Payload,
        S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    {
        type ReqBody = ReqBody;
        type ResBody = ResBody;
        type Error = S::Error;
        type Service = HttpService<S::Service>;
        type InitError = S::InitError;
        #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
        type Future = future::Map<S::Future, fn(S::Service) -> HttpService<S::Service>>;

        #[inline]
        fn new_service(&self) -> Self::Future {
            self.0.new_service().map(HttpService)
        }
    }

    #[derive(Debug)]
    struct HttpService<S>(S);

    impl<S, ReqBody, ResBody> hyper_service::Service for HttpService<S>
    where
        S: tower_service::Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
        ReqBody: Payload,
        ResBody: Payload,
        S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    {
        type ReqBody = ReqBody;
        type ResBody = ResBody;
        type Error = S::Error;
        type Future = S::Future;

        #[inline]
        fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
            self.0.call(request)
        }
    }
}
