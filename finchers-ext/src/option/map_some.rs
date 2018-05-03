#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, Poll};

#[derive(Debug, Copy, Clone)]
pub struct MapSome<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, T>(endpoint: E, f: F) -> MapSome<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone + Send + Sync,
{
    MapSome { endpoint, f }
}

impl<E, F, T, U> Endpoint for MapSome<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone + Send + Sync,
{
    type Output = Option<U>;
    type Task = MapSomeTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(MapSomeTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapSomeTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F, A, U> Task for MapSomeTask<T, F>
where
    T: Task<Output = Option<A>> + Send,
    F: FnOnce(A) -> U + Send,
{
    type Output = Option<U>;

    fn poll_task(&mut self, cx: &mut task::Context) -> Poll<Result<Self::Output, Error>> {
        self.task.poll_task(cx).map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            cx.input().enter_scope(|| item.map(f))
        })
    }
}
