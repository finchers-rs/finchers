use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, PollTask, Task};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint<Item = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send,
{
    AndThen { endpoint, f }
}

impl<E, F, A, B, U> Endpoint for AndThen<E, F>
where
    E: Endpoint<Item = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send,
{
    type Item = Result<U, B>;
    type Task = AndThenTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(AndThenTask {
            task,
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

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| Ok(item.and_then(f).into()))
    }
}
