#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, PollResult};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone,
{
    AndThen { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for AndThen<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone,
{
    type Output = Result<U, B>;
    type Task = AndThenTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(AndThenTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AndThenTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Task for AndThenTask<T, F>
where
    T: Task<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B>,
{
    type Output = Result<U, B>;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.and_then(f)
        })
    }
}
