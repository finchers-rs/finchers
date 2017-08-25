#![allow(missing_docs)]

use futures::future::{self, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointResult};
use errors::FinchersError;


/// Create an endpoint which returns a success value of `T`
pub fn ok<T>(x: T) -> EndpointOk<T> {
    EndpointOk(x)
}


/// The return type of `ok(x)`
pub struct EndpointOk<T>(T);

impl<T> Endpoint for EndpointOk<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply(self, _: &mut Context) -> EndpointResult<Self::Future> {
        Ok(future::ok(self.0))
    }
}
