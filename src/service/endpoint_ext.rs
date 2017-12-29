use std::fmt;
use std::error::Error;
use tokio_core::reactor::Handle;
use endpoint::Endpoint;
use http::{CookieManager, StatusCode};
use responder::{ErrorResponder, IntoResponder};
use super::EndpointService;

/// An error represents which represents that
/// the matched route was not found.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("not found")
    }
}

impl Error for NoRoute {
    fn description(&self) -> &str {
        "not found"
    }
}

impl ErrorResponder for NoRoute {
    fn status(&self) -> StatusCode {
        StatusCode::NotFound
    }

    fn message(&self) -> Option<String> {
        None
    }
}

///
pub trait EndpointServiceExt: Endpoint + Clone
where
    Self::Item: IntoResponder,
    Self::Error: IntoResponder,
{
    /// Create a new `Service` from this endpoint
    fn to_service(&self, handle: Handle, cookie_manager: CookieManager, no_route: NoRoute) -> EndpointService<Self> {
        EndpointService {
            endpoint: self.clone(),
            handle,
            cookie_manager,
            no_route,
        }
    }
}

impl<E: Endpoint + Clone> EndpointServiceExt for E
where
    E::Item: IntoResponder,
    E::Error: IntoResponder,
{
}
