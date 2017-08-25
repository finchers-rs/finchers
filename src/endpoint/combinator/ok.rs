#![allow(missing_docs)]

use futures::future::{self, FutureResult};

use context::Context;
use endpoint::{Endpoint,EndpointResult};
use errors::FinchersError;


#[allow(missing_docs)]
pub struct EndpointOk<T>(T);

impl<T> Endpoint for EndpointOk<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply(self, _: &mut Context) -> EndpointResult<Self::Future> {
        Ok(future::ok(self.0))
    }
}

#[allow(missing_docs)]
pub fn ok<T>(x: T) -> EndpointOk<T> {
    EndpointOk(x)
}
