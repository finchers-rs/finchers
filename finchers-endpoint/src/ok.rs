#![allow(missing_docs)]

use finchers_core::error::Error;
use finchers_core::request::Input;
use futures::future::{self, FutureResult};
use {Context, Endpoint};

pub fn ok<T: Clone>(x: T) -> EndpointOk<T> {
    EndpointOk { x }
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointOk<T> {
    x: T,
}

impl<T: Clone> Endpoint for EndpointOk<T> {
    type Item = T;
    type Future = FutureResult<T, Error>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(future::ok(self.x.clone()))
    }
}
