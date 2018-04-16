#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint, Error};
use finchers_core::task::CompatTask;
use futures::future::{self, FutureResult};

pub fn ok<T>(x: T) -> Ok<T>
where
    T: Clone + Send,
{
    Ok { x }
}

#[derive(Debug, Clone, Copy)]
pub struct Ok<T> {
    x: T,
}

impl<T> Endpoint for Ok<T>
where
    T: Clone + Send,
{
    type Item = T;
    type Task = CompatTask<FutureResult<T, Error>>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(future::ok(self.x.clone()).into())
    }
}
