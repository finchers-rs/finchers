#![allow(missing_docs)]

use std::marker::PhantomData;

use futures::future::{self, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Create an endpoint which returns a success value of `T`
pub fn ok<T, E>(x: T) -> EndpointOk<T, E> {
    EndpointOk(x, PhantomData)
}


/// The return type of `ok(x)`
#[derive(Debug)]
pub struct EndpointOk<T, E>(T, PhantomData<fn() -> E>);

impl<T, E> Endpoint for EndpointOk<T, E> {
    type Item = T;
    type Error = E;
    type Future = FutureResult<T, E>;

    fn apply(self, _: &mut Context) -> EndpointResult<Self::Future> {
        Ok(future::ok(self.0))
    }
}
