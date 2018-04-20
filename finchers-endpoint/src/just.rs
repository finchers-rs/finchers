#![allow(missing_docs)]

use finchers_core::endpoint::Context;
use finchers_core::task::CompatTask;
use finchers_core::{Endpoint, Error};
use futures::future::{self, FutureResult};

pub fn just<T>(x: T) -> Just<T>
where
    T: Clone + Send,
{
    Just { x }
}

#[derive(Debug, Clone, Copy)]
pub struct Just<T> {
    x: T,
}

impl<T> Endpoint for Just<T>
where
    T: Clone + Send,
{
    type Item = T;
    type Task = CompatTask<FutureResult<T, Error>>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(future::ok(self.x.clone()).into())
    }
}
