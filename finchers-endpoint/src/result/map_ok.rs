use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, PollTask, Task};

#[derive(Debug, Copy, Clone)]
pub struct MapOk<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapOk<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone + Send,
{
    MapOk { endpoint, f }
}

impl<E, F, A, B, U> Endpoint for MapOk<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone + Send,
{
    type Output = Result<U, B>;
    type Task = MapOkTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(MapOkTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapOkTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Task for MapOkTask<T, F>
where
    T: Task<Output = Result<A, B>> + Send,
    F: FnOnce(A) -> U + Send,
{
    type Output = Result<U, B>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        self.task.poll_task(cx).map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            cx.input().enter_scope(|| item.map(f))
        })
    }
}
