//! The definition of middleware layer.
//!
//! # Note
//! The components in this module is intentionally have the same signature
//! to the traits in `tower-service` and `tower-web::middleware` and they
//! will be completely replaced with their definitions in the future version.

use std::sync::Arc;

use futures::{Future, Poll};
use http::{Request, Response};

#[doc(no_inline)]
pub use tower_service::{NewService, Service};

pub trait Middleware<S> {
    type Request;
    type Response;
    type Error;
    type Service: Service<Request = Self::Request, Response = Self::Response, Error = Self::Error>;

    fn wrap(&self, input: S) -> Self::Service;
}

impl<M, S> Middleware<S> for Arc<M>
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
    S: NewService,
    M: Middleware<S::Service> + Clone,
{
    pub fn new(new_service: S, middleware: M) -> Self {
        Chain {
            new_service,
            middleware,
        }
    }
}

impl<S, M> NewService for Chain<S, M>
where
    S: NewService,
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
    F::Item: Service,
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

// ---- Tower Web ----

#[cfg(feature = "tower-web")]
pub use self::imp_tower_web_integration::arced;
#[cfg(feature = "tower-web")]
pub(crate) use self::imp_tower_web_integration::TowerWebMiddleware;

#[cfg(feature = "tower-web")]
mod imp_tower_web_integration {
    use super::Middleware;
    use std::sync::Arc;
    use tower_service;
    use tower_web;

    #[derive(Debug, Copy, Clone)]
    pub struct TowerWebMiddleware<M>(M);

    impl<M> TowerWebMiddleware<M> {
        pub fn new(middleware: M) -> TowerWebMiddleware<M> {
            TowerWebMiddleware(middleware)
        }
    }

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

    pub fn arced<M>(middleware: M) -> Arced<M> {
        Arced(Arc::new(middleware))
    }

    #[derive(Debug)]
    pub struct Arced<M>(Arc<M>);

    impl<M> Clone for Arced<M> {
        fn clone(&self) -> Self {
            Arced(self.0.clone())
        }
    }

    impl<M, S> tower_web::middleware::Middleware<S> for Arced<M>
    where
        M: tower_web::middleware::Middleware<S>,
    {
        type Request = M::Request;
        type Response = M::Response;
        type Error = M::Error;
        type Service = M::Service;

        fn wrap(&self, input: S) -> Self::Service {
            self.0.wrap(input)
        }
    }
}

pub fn map_response<F>(f: F) -> MapResponse<F> {
    MapResponse(f)
}

#[derive(Debug)]
pub struct MapResponse<F>(F);

impl<S, ReqBody, ResBodyT, ResBodyU, F> Middleware<S> for MapResponse<F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU + Clone,
{
    type Request = Request<ReqBody>;
    type Response = Response<ResBodyU>;
    type Error = S::Error;
    type Service = MapResponseService<S, F>;

    fn wrap(&self, inner: S) -> Self::Service {
        MapResponseService {
            inner,
            f: self.0.clone(),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct MapResponseService<S, F> {
    inner: S,
    f: F,
}

impl<S, ReqBody, ResBodyT, ResBodyU, F> Service for MapResponseService<S, F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU + Clone,
{
    type Request = Request<ReqBody>;
    type Response = Response<ResBodyU>;
    type Error = S::Error;
    type Future = MapResponseServiceFuture<S::Future, F>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.inner.poll_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        MapResponseServiceFuture {
            future: self.inner.call(request),
            f: Some(self.f.clone()),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct MapResponseServiceFuture<Fut, F> {
    future: Fut,
    f: Option<F>,
}

impl<Fut, ResBodyT, ResBodyU, F> Future for MapResponseServiceFuture<Fut, F>
where
    Fut: Future<Item = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU,
{
    type Item = Response<ResBodyU>;
    type Error = Fut::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map(|x| {
            x.map(|res| {
                let f = self.f.take().expect("The future has already polled");
                res.map(f)
            })
        })
    }
}
