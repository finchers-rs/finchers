use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, PollTask, Task};

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapErr<E, F>
where
    E: Endpoint<Item = Result<A, B>>,
    F: FnOnce(B) -> U + Clone + Send,
{
    MapErr { endpoint, f }
}

impl<E, F, A, B, U> Endpoint for MapErr<E, F>
where
    E: Endpoint<Item = Result<A, B>>,
    F: FnOnce(B) -> U + Clone + Send,
{
    type Item = Result<A, U>;
    type Task = MapErrTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(MapErrTask {
            task,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapErrTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Task for MapErrTask<T, F>
where
    T: Task<Output = Result<A, B>> + Send,
    F: FnOnce(B) -> U + Send,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| Ok(item.map_err(f).into()))
    }
}
