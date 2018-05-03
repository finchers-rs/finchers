#![allow(missing_docs)]

use finchers_core::Endpoint;
use finchers_core::endpoint::Context;
use finchers_core::task;

/// Create an endpoint which applies the given function to the incoming request and returns
/// an immediate value of `T`.
///
/// NOTE: The trait bound of returned value from `F` should be replaced with `IntoTask`.
pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: Fn(&mut Context) -> Option<T> + Send + Sync,
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
    F: Fn(&mut Context) -> Option<T> + Send + Sync,
    T: Send,
{
    type Output = T;
    type Task = task::Ready<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (self.f)(cx).map(task::ready)
    }
}
