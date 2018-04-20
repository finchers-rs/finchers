#![allow(missing_docs)]

use finchers_core::endpoint::Context;
use finchers_core::task::CompatTask;
use finchers_core::{Endpoint, Error};
use futures::future::{self, FutureResult};

pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: Fn(&mut Context) -> Option<T>,
    T: Send,
{
    Lazy { f }
}

#[derive(Debug, Clone, Copy)]
pub struct Lazy<F> {
    f: F,
}

impl<F, T> Endpoint for Lazy<F>
where
    F: Fn(&mut Context) -> Option<T>,
    T: Send,
{
    type Item = T;
    type Task = CompatTask<FutureResult<T, Error>>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (self.f)(cx).map(|t| CompatTask::from(future::ok(t)))
    }
}
