use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, Error, IntoEndpoint, task::{self, PollTask, Task}};

pub fn new<E>(endpoint: E) -> Abort<E::Endpoint>
where
    E: IntoEndpoint,
    E::Item: HttpError,
{
    Abort {
        endpoint: endpoint.into_endpoint(),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Abort<E> {
    endpoint: E,
}

impl<E> Endpoint for Abort<E>
where
    E: Endpoint,
    E::Item: HttpError,
{
    type Item = !;
    type Task = AbortTask<E::Task>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(AbortTask { task })
    }
}

#[derive(Debug)]
pub struct AbortTask<T> {
    task: T,
}

impl<T: Task> Task for AbortTask<T>
where
    T::Output: HttpError,
{
    type Output = !;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        Err(Error::from(item).into())
    }
}
