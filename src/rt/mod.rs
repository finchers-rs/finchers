//! **[unstable]**
//! The experimental implementation of new runtime.
//!
//! # Compatibility Notes
//! This module is disabled by default and some breaking changes
//! may be occurred without bumping the major version.

pub(crate) mod blocking;
mod server;

pub mod app;
pub mod middleware;
pub mod test;

// re-exports
pub use self::blocking::{blocking, blocking_section, BlockingSection};
pub use self::middleware::Middleware;
#[doc(no_inline)]
pub use tokio::executor::DefaultExecutor;
#[doc(no_inline)]
pub use tokio::spawn;

// ==== ServiceBuilder ====

/// A builder of HTTP server.
#[derive(Debug)]
pub struct ServiceBuilder<S> {
    new_service: S,
}

mod service_builder {
    use super::middleware::{Chain, Middleware};
    use super::ServiceBuilder;
    use tower_service::NewService;

    #[cfg(feature = "tower-web")]
    use super::middleware::TowerWebMiddleware;
    #[cfg(feature = "tower-web")]
    use tower_web;

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
}

// ==== launch ====

use self::app::{App, IsAppEndpoint};

/// Create an instance of `ServiceBuilder` from the specified endpoint.
pub fn launch<E>(endpoint: E) -> ServiceBuilder<App<E>>
where
    E: IsAppEndpoint,
{
    ServiceBuilder::new(App::new(endpoint))
}

// ==== spawn_with_handle ====

use futures::sync::oneshot;
use futures::sync::oneshot::SpawnHandle;
use futures::Future;

/// Spawns a future onto the default executor and returns its handle.
#[inline]
pub fn spawn_with_handle<F>(future: F) -> SpawnHandle<F::Item, F::Error>
where
    F: Future + Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    oneshot::spawn(future, &DefaultExecutor::current())
}
