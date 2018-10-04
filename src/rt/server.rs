//! The implementation of HTTP server based on hyper and tower-service.

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
#[cfg(feature = "tower-web")]
use tower_web;

use super::middleware::{Chain, Middleware, TowerWebMiddleware};

/// A builder of HTTP server.
#[derive(Debug)]
pub struct ServerBuilder<S> {
    new_service: S,
}

impl<S> ServerBuilder<S> {
    /// Creates a new `ServerBuilder` from the specified NewService.
    pub fn new(new_service: S) -> ServerBuilder<S> {
        ServerBuilder { new_service }
    }

    /// Wraps the inner service into the specified middleware.
    pub fn with_middleware<M>(self, middleware: M) -> ServerBuilder<Chain<S, M>>
    where
        S: tower_service::NewService,
        M: Middleware<S::Service> + Clone,
    {
        ServerBuilder {
            new_service: Chain::new(self.new_service, middleware),
        }
    }

    /// Wraps the inner service into the specified Tower-web middleware.
    #[cfg(feature = "tower-web")]
    pub fn with_tower_middleware<M>(
        self,
        middleware: M,
    ) -> ServerBuilder<Chain<S, Arc<TowerWebMiddleware<M>>>>
    where
        S: tower_service::NewService,
        M: tower_web::middleware::Middleware<S::Service>,
    {
        ServerBuilder {
            new_service: Chain::new(
                self.new_service,
                Arc::new(TowerWebMiddleware::new(middleware)),
            ),
        }
    }

    /// Consumes itself and returns the inner `NewService`.
    pub fn into_tower_service(self) -> S {
        self.new_service
    }

    /// Start the server with binding the specified listener address.
    pub fn serve<Bd>(self, addr: impl ToSocketAddrs) -> Fallible<()>
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
        let mut addrs = addr.to_socket_addrs()?;
        let addr = addrs
            .next()
            .ok_or_else(|| ::failure::err_msg("invalid listener addr"))?;
        serve_with_shutdown_signal(
            Server::try_bind(&addr)?,
            Runtime::new()?,
            self.new_service,
            future::empty::<(), ()>(),
        )
    }
}

/// Starts an HTTP server with the specified components.
pub fn serve_with_shutdown_signal<I, S, Bd>(
    builder: Builder<I>,
    mut rt: Runtime,
    new_service: S,
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

    let server = builder
        .serve(NewHttpService(new_service.clone()))
        .with_graceful_shutdown(signal)
        .map_err(|err| error!("server error: {}", err));

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
