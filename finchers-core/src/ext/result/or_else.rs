#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, PollResult};

#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> OrElse<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U> + Clone,
{
    OrElse { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for OrElse<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U> + Clone,
{
    type Output = Result<A, U>;
    type Task = OrElseTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(OrElseTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OrElseTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Task for OrElseTask<T, F>
where
    T: Task<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U>,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.or_else(f)
        })
    }
}
