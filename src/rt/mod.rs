//! **[unstable]**
//! The experimental implementation of new runtime.
//!
//! # Compatibility Notes
//! This module is disabled by default and some breaking changes
//! may be occurred without bumping the major version.

#![allow(missing_docs)]

pub mod app;
pub mod blocking;
pub mod middleware;
pub mod server;
pub mod testing;

// re-exports
pub use self::app::{App, IsAppEndpoint};
pub use self::blocking::{blocking, blocking_section};
pub use self::middleware::Middleware;
pub use self::server::ServerBuilder;

/// Create an instance of `ServerBuilder` from the specified endpoint.
pub fn launch<E>(endpoint: E) -> ServerBuilder<App<E>>
where
    E: IsAppEndpoint,
{
    ServerBuilder::new(App::new(endpoint))
}
