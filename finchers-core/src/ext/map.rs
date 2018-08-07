#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::task::Task;
use crate::{Error, PollResult};

pub fn new<E, F, T>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Output) -> T + Clone,
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

impl<E, F, T> EndpointBase for Map<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Output) -> T + Clone,
{
    type Output = F::Output;
    type Task = MapTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(MapTask {
            task: self.endpoint.apply(cx)?,
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
    T: Task,
    F: FnOnce(T::Output) -> U,
{
    type Output = U;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            f(item)
        })
    }
}
