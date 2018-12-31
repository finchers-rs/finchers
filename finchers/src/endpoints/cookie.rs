//! Endpoints for managing Cookie values.

use futures::future;

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::input::Cookies;

/// Creates an endpoint which returns an object for tracking Cookie values.
///
/// # Example
///
/// ```
/// # extern crate finchers;
/// # use finchers::prelude::*;
/// # use finchers::endpoints::cookie::cookies;
/// # use finchers::input::Cookies;
/// #
/// # fn main() {
/// let endpoint = cookies()
///     .map(|cookies: Cookies| {
///         let session_id = cookies.get("session-id");
///         // ...
/// #       drop(session_id);
/// #       ()
///     });
/// # drop(endpoint);
/// # }
/// ```
pub fn cookies() -> CookiesEndpoint {
    CookiesEndpoint { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CookiesEndpoint {
    _priv: (),
}

impl Endpoint for CookiesEndpoint {
    type Output = (Cookies,);
    type Future = future::FutureResult<Self::Output, Error>;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(future::result(
            cx.input().cookies().map(|cookies| (cookies,)),
        ))
    }
}
