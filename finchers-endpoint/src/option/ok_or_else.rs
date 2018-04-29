use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, PollTask, Task};

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, T, U>(endpoint: E, f: F) -> OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send,
{
    OkOrElse { endpoint, f }
}

impl<E, F, T, U> Endpoint for OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send,
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

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        self.task.poll_task(cx).map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            cx.input().enter_scope(|| item.ok_or_else(f))
        })
    }
}
