use std::error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use failure::Fallible;
use futures::future;
use futures::{Future, Stream};
use http::{Request, Response};
use hyper;
use hyper::body::Payload;
use hyper::server::conn::AddrIncoming;
use hyper::Body;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::runtime::Runtime;
use tower_service::{NewService, Service};

use rt::{with_set_runtime_mode, RuntimeMode};

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

    fn build(self) -> Fallible<HttpServer<Self::Incoming>>;
}

impl<'a> ServerConfigImpl for &'a str {
    type Item = <AddrIncoming as Stream>::Item;
    type Error = <AddrIncoming as Stream>::Error;
    type Incoming = AddrIncoming;

    fn build(self) -> Fallible<HttpServer<Self::Incoming>> {
        ServerConfigImpl::build(&self.parse::<SocketAddr>()?)
    }
}

impl ServerConfigImpl for String {
    type Item = <AddrIncoming as Stream>::Item;
    type Error = <AddrIncoming as Stream>::Error;
    type Incoming = AddrIncoming;

    fn build(self) -> Fallible<HttpServer<Self::Incoming>> {
        ServerConfigImpl::build(self.as_str())
    }
}

impl<I: Into<IpAddr>> ServerConfigImpl for (I, u16) {
    type Item = <AddrIncoming as Stream>::Item;
    type Error = <AddrIncoming as Stream>::Error;
    type Incoming = AddrIncoming;

    fn build(self) -> Fallible<HttpServer<Self::Incoming>> {
        ServerConfigImpl::build(SocketAddr::from(self))
    }
}

impl ServerConfigImpl for SocketAddr {
    type Item = <AddrIncoming as Stream>::Item;
    type Error = <AddrIncoming as Stream>::Error;
    type Incoming = AddrIncoming;

    fn build(self) -> Fallible<HttpServer<Self::Incoming>> {
        ServerConfigImpl::build(&self)
    }
}

impl<'a> ServerConfigImpl for &'a SocketAddr {
    type Item = <AddrIncoming as Stream>::Item;
    type Error = <AddrIncoming as Stream>::Error;
    type Incoming = AddrIncoming;

    fn build(self) -> Fallible<HttpServer<Self::Incoming>> {
        let builder = hyper::server::Server::try_bind(self)?;
        let rt = Runtime::new()?;
        Ok(HttpServer { builder, rt })
    }
}

#[derive(Debug)]
pub struct HttpServer<I> {
    builder: hyper::server::Builder<I>,
    rt: Runtime,
}

impl<I> HttpServer<I>
where
    I: Stream + Send + 'static,
    I::Item: AsyncRead + AsyncWrite + Send + 'static,
    I::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
{
    pub(super) fn serve_with_graceful_shutdown<S, Bd>(
        self,
        new_service: S,
        signal: impl Future<Item = ()> + Send + 'static,
    ) where
        S: NewService<Request = Request<Body>, Response = Response<Bd>> + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        S::InitError: Into<Box<dyn error::Error + Send + Sync + 'static>>,
        S::Service: Send,
        S::Future: Send + 'static,
        <S::Service as Service>::Future: Send + 'static,
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
    }
}

#[derive(Debug)]
struct NewHttpService<S>(S);

impl<S, ReqBody, ResBody> hyper::service::NewService for NewHttpService<S>
where
    S: NewService<Request = Request<ReqBody>, Response = Response<ResBody>>,
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

impl<S, ReqBody, ResBody> hyper::service::Service for HttpService<S>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
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
