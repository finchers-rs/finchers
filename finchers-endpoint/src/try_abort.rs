use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, task::{self, PollTask, Task}};

pub fn new<E, F, T, R>(endpoint: E, f: F) -> TryAbort<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Result<T, R> + Clone + Send,
    R: HttpError,
{
    TryAbort { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct TryAbort<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, T, R> Endpoint for TryAbort<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Result<T, R> + Clone + Send,
    R: HttpError,
{
    type Item = T;
    type Task = TryAbortTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(TryAbortTask {
            task,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct TryAbortTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U, E> Task for TryAbortTask<T, F>
where
    T: Task + Send,
    F: FnOnce(T::Output) -> Result<U, E> + Clone + Send,
    E: HttpError,
{
    type Output = U;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        let f = self.f.take().expect("cannot resolve/reject twice");
        f(item).map_err(Into::into).map(Into::into)
    }
}
