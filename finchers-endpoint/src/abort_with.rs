use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint, task::{self, PollTask, Task}};

pub fn new<E, F, U>(endpoint: E, f: F) -> AbortWith<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Item) -> U + Clone + Send,
    U: HttpError,
{
    AbortWith {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbortWith<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, U> Endpoint for AbortWith<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> U + Clone + Send,
    U: HttpError,
{
    type Item = !;
    type Task = AbortWithTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(AbortWithTask {
            task,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AbortWithTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U> Task for AbortWithTask<T, F>
where
    T: Task + Send,
    F: FnOnce(T::Output) -> U + Send,
    U: HttpError,
{
    type Output = !;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        let f = self.f.take().expect("cannot resolve twice");
        Err(f(item).into())
    }
}
