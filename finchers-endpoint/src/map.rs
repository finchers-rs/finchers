use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint, task::{self, PollTask, Task}};

pub fn new<E, F, T>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Item) -> T + Clone + Send,
{
    Map {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Map<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, T> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> T + Clone + Send,
{
    type Item = F::Output;
    type Task = MapTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(MapTask {
            task,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U> Task for MapTask<T, F>
where
    T: Task + Send,
    F: FnOnce(T::Output) -> U + Send,
{
    type Output = U;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        let f = self.f.take().expect("cannot resolve twice");
        Ok(f(item).into())
    }
}
