use futures_util::future;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

/// Create an endpoint which simply returns an unit (`()`).
pub fn unit() -> Unit {
    Unit { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Unit {
    _priv: (),
}

impl Endpoint for Unit {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(future::ready(Ok(())))
    }
}
