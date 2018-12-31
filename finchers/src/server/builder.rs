use std::error;

use futures::{future, Future};
use http::{Request, Response};
use hyper::body::{Body, Payload};
use tower_service;
use tower_service::NewService;

use super::error::{ServerError, ServerResult};
use super::http_server::ServerConfig;

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
            .map_err(ServerError::config)?
            .serve_with_graceful_shutdown(self, signal);
        Ok(())
    }
}
