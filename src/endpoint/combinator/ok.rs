#![allow(missing_docs)]

use futures::future::{self, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointResult};
use util::NoReturn;


/// Create an endpoint which returns a success value of `T`
pub fn ok<T>(x: T) -> EndpointOk<T> {
    EndpointOk(x)
}


/// The return type of `ok(x)`
#[derive(Debug)]
pub struct EndpointOk<T>(T);

impl<T> Endpoint for EndpointOk<T> {
    type Item = T;
    type Error = NoReturn;
    type Future = FutureResult<T, NoReturn>;

    fn apply(self, _: &mut Context) -> EndpointResult<Self::Future> {
        Ok(future::ok(self.0))
    }
}

