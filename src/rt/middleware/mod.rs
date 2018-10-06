//! The definition of middleware layer.
//!
//! # Note
//! The components in this module is intentionally have the same signature
//! to the traits in `tower-service` and `tower-web::middleware` and they
//! will be completely replaced with their definitions in the future version.

#![allow(missing_docs)]

pub mod log;
mod map_response_body;

// ---- reexports ----
pub use self::map_response_body::{map_response_body, MapResponseBody};
#[doc(no_inline)]
pub use tower_service::{NewService, Service};

// ---- Middleware ----

use futures::{Future, Poll};
use std::sync::Arc;

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
    type Future = self::imp_chain::ChainFuture<S::Future, M>;
    type InitError = S::InitError;

    fn new_service(&self) -> Self::Future {
        self::imp_chain::ChainFuture {
            future: self.new_service.new_service(),
            middleware: self.middleware.clone(),
        }
    }
}

mod imp_chain {
    use super::*;

    #[derive(Debug)]
    pub struct ChainFuture<F, M> {
        pub(super) future: F,
        pub(super) middleware: M,
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
}

// ---- Tower Web ----

#[cfg(feature = "tower-web")]
pub(crate) use self::imp_tower_web_integration::TowerWebMiddleware;

#[cfg(feature = "tower-web")]
mod imp_tower_web_integration {
    use super::Middleware;
    use std::sync::Arc;
    use tower_service;
    use tower_web;

    #[derive(Debug)]
    pub struct TowerWebMiddleware<M>(Arc<M>);

    impl<M> Clone for TowerWebMiddleware<M> {
        fn clone(&self) -> Self {
            TowerWebMiddleware(self.0.clone())
        }
    }

    impl<M> TowerWebMiddleware<M> {
        pub fn new(middleware: M) -> TowerWebMiddleware<M> {
            TowerWebMiddleware(Arc::new(middleware))
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
}
