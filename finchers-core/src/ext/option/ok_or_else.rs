#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::poll::Poll;
use crate::task::Task;

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, T, U>(endpoint: E, f: F) -> OkOrElse<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce() -> U + Clone,
{
    OkOrElse { endpoint, f }
}

impl<E, F, T, U> EndpointBase for OkOrElse<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce() -> U + Clone,
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
    T: Task<Output = Option<A>>,
    F: FnOnce() -> U,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self) -> Poll<Self::Output> {
        self.task.poll_task().map(|item: Option<A>| {
            let f = self.f.take().expect("cannot resolve twice");
            item.ok_or_else(f)
        })
    }
}
