use std::error;
use std::sync::Arc;

use futures::{future, Future, Poll};
use http::{Request, Response};
use hyper::body::Payload;
use hyper::service as hyper_service;
use tower_service;

#[derive(Debug)]
pub struct NewHttpService<S>(S);

impl<S> NewHttpService<S> {
    pub(super) fn new(new_service: S) -> NewHttpService<S> {
        NewHttpService(new_service)
    }
}

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

    fn new_service(&self) -> Self::Future {
        self.0.new_service().map(HttpService)
    }
}

#[derive(Debug)]
pub struct HttpService<S>(S);

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

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        self.0.call(request)
    }
}

pub trait Middleware<S: tower_service::Service> {
    type Request;
    type Response;
    type Error;
    type Service: tower_service::Service<
        Request = Self::Request,
        Response = Self::Response,
        Error = Self::Error,
    >;

    fn wrap(&self, input: S) -> Self::Service;
}

impl<M, S: tower_service::Service> Middleware<S> for Arc<M>
where
    M: Middleware<S>,
{
    type Request = M::Request;
    type Response = M::Response;
    type Error = M::Error;
    type Service = M::Service;

    fn wrap(&self, input: S) -> Self::Service {
        (**self).wrap(input)
    }
}

#[derive(Debug)]
pub struct Chain<S, M> {
    new_service: S,
    middleware: M,
}

impl<S, M> Chain<S, M>
where
    S: tower_service::NewService,
    M: Middleware<S::Service> + Clone,
{
    pub fn new(new_service: S, middleware: M) -> Self {
        Chain {
            new_service,
            middleware,
        }
    }
}

impl<S, M> tower_service::NewService for Chain<S, M>
where
    S: tower_service::NewService,
    M: Middleware<S::Service> + Clone,
{
    type Request = M::Request;
    type Response = M::Response;
    type Error = M::Error;
    type Service = M::Service;
    type Future = ChainFuture<S::Future, M>;
    type InitError = S::InitError;

    fn new_service(&self) -> Self::Future {
        ChainFuture {
            future: self.new_service.new_service(),
            middleware: self.middleware.clone(),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ChainFuture<F, M> {
    future: F,
    middleware: M,
}

impl<F, M> Future for ChainFuture<F, M>
where
    F: Future,
    F::Item: tower_service::Service,
    M: Middleware<F::Item>,
{
    type Item = M::Service;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future
            .poll()
            .map(|x| x.map(|service| self.middleware.wrap(service)))
    }
}

#[cfg(feature = "tower-web")]
pub use self::imp_tower_web_integration::TowerWebMiddleware;

#[cfg(feature = "tower-web")]
mod imp_tower_web_integration {
    use super::Middleware;
    use tower_service;
    use tower_web;

    #[derive(Debug, Copy, Clone)]
    pub struct TowerWebMiddleware<M>(M);

    impl<M> TowerWebMiddleware<M> {
        pub fn new(middleware: M) -> TowerWebMiddleware<M> {
            TowerWebMiddleware(middleware)
        }
    }

    #[cfg(feature = "tower-web")]
    impl<M, S> Middleware<S> for TowerWebMiddleware<M>
    where
        M: tower_web::middleware::Middleware<S>,
        S: tower_service::Service,
    {
        type Request = M::Request;
        type Response = M::Response;
        type Error = M::Error;
        type Service = M::Service;

        #[inline]
        fn wrap(&self, inner: S) -> Self::Service {
            self.0.wrap(inner)
        }
    }
}
