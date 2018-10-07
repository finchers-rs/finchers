//! The implementation of HTTP server based on hyper and tower-service.

pub mod middleware;

use std::error;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use failure::Fallible;
use futures::{future, Future, Stream};
use http::{Request, Response};
use hyper::body::{Body, Payload};
use hyper::server::{Builder, Server};
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
use self::middleware::TowerWebMiddleware;
use self::middleware::{Chain, Middleware};

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
    S: tower_service::NewService<Request = Request<Body>, Response = Response<Bd>>
        + Send
        + Sync
        + 'static,
    S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    S::Service: Send,
    S::Future: Send + 'static,
    <S::Service as tower_service::Service>::Future: Send + 'static,
    Bd: Payload,
{
    /// Start the server with binding the specified listener address.
    pub fn serve_http(self, addr: impl ToSocketAddrs) -> Fallible<()> {
        let mut addrs = addr.to_socket_addrs()?;
        let addr = addrs
            .next()
            .ok_or_else(|| ::failure::err_msg("invalid listener addr"))?;
        self.serve_with_config(
            Server::try_bind(&addr)?,
            Runtime::new()?,
            future::empty::<(), ()>(),
        )
    }

    /// Start the server using the specified components.
    pub fn serve_with_config<I, F>(
        self,
        builder: Builder<I>,
        rt: Runtime,
        signal: F,
    ) -> Fallible<()>
    where
        I: Stream + Send + 'static,
        I::Item: AsyncRead + AsyncWrite + Send + 'static,
        I::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        F: Future<Item = ()> + Send + 'static,
    {
        serve(self, builder, rt, signal)
    }
}

fn serve<I, S, Bd>(
    new_service: S,
    builder: Builder<I>,
    mut rt: Runtime,
    signal: impl Future<Item = ()> + Send + 'static,
) -> Fallible<()>
where
    I: Stream + Send + 'static,
    I::Item: AsyncRead + AsyncWrite + Send + 'static,
    I::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    S: tower_service::NewService<Request = Request<Body>, Response = Response<Bd>>
        + Send
        + Sync
        + 'static,
    S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    S::Service: Send,
    S::Future: Send + 'static,
    <S::Service as tower_service::Service>::Future: Send + 'static,
    Bd: Payload,
{
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

    let server =
        future::poll_fn(move || with_set_runtime_mode(RuntimeMode::ThreadPool, || server.poll()));

    rt.spawn(server);
    rt.shutdown_on_idle().wait().unwrap();
    Ok(())
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
