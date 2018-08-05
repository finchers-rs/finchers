#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::Task;
use finchers_core::{Error, Poll};

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, T, U>(endpoint: E, f: F) -> OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send + Sync,
{
    OkOrElse { endpoint, f }
}

impl<E, F, T, U> Endpoint for OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send + Sync,
{
    type Output = Result<T, U>;
    type Task = OkOrElseTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(OkOrElseTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OkOrElseTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, A, U> Task for OkOrElseTask<T, F>
where
    T: Task<Output = Option<A>> + Send,
    F: FnOnce() -> U + Send,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self) -> Poll<Result<Self::Output, Error>> {
        self.task.poll_task().map_ok(|item: Option<A>| {
            let f = self.f.take().expect("cannot resolve twice");
            item.ok_or_else(f)
        })
    }
}
