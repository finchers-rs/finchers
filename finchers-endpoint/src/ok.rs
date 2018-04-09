#![allow(missing_docs)]

use finchers_core::Input;
use finchers_core::endpoint::{Context, Endpoint, Error};
use futures::future::{self, FutureResult};

pub fn ok<T: Clone>(x: T) -> Ok<T> {
    Ok { x }
}

#[derive(Debug, Clone, Copy)]
pub struct Ok<T> {
    x: T,
}

impl<T: Clone> Endpoint for Ok<T> {
    type Item = T;
    type Future = FutureResult<T, Error>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(future::ok(self.x.clone()))
    }
}
