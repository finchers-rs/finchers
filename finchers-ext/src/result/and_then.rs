#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, PollResult};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send + Sync,
{
    AndThen { endpoint, f }
}

impl<E, F, A, B, U> Endpoint for AndThen<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send + Sync,
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
    T: Task<Output = Result<A, B>> + Send,
    F: FnOnce(A) -> Result<U, B> + Send,
{
    type Output = Result<U, B>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        self.task.poll_task(cx).map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            cx.input().enter_scope(|| item.and_then(f))
        })
    }
}
