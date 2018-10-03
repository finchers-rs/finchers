//! **[unstable]**
//! The experimental implementation of new runtime.
//!
//! # Compatibility Notes
//! This module is disabled by default and some breaking changes
//! may be occurred without bumping the major version.

#![allow(missing_docs)]

mod app;
pub mod server;
pub mod service;
pub mod testing;

// re-exports
pub use self::app::{App, IsAppEndpoint};
pub use self::server::ServerBuilder;
pub use self::service::Middleware;
#[cfg(feature = "tower-web")]
pub use self::service::TowerWebMiddleware;

#[doc(no_inline)]
pub use tokio_threadpool::blocking;

// === impl ====

use futures::future::poll_fn;
use futures::{Async, Future};

use error::fail;
use error::Error;

/// A helper function to create a future from a blocking section.
///
/// # Example
///
/// ```ignore
/// path!(@get / u32 /)
///     .and_then(|id: u32| blocking_section(|| {
///         get_post_sync(id).map_err(finchers::error::fail)
///     }))
pub fn blocking_section<T>(
    f: impl FnOnce() -> Result<T, Error>,
) -> impl Future<Item = T, Error = Error> {
    let mut f_opt = Some(f);
    poll_fn(move || {
        try_ready!(blocking(|| (f_opt.take().unwrap())()).map_err(fail)).map(Async::Ready)
    })
}

/// Create an instance of `ServerBuilder` from the specified endpoint.
pub fn launch<E>(endpoint: E) -> ServerBuilder<App<E>>
where
    E: IsAppEndpoint,
{
    ServerBuilder::new(App::new(endpoint))
}
