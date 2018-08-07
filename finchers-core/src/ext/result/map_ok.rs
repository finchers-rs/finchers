#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, PollResult};

#[derive(Debug, Copy, Clone)]
pub struct MapOk<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapOk<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone,
{
    MapOk { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for MapOk<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone,
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
    T: Task<Output = Result<A, B>>,
    F: FnOnce(A) -> U,
{
    type Output = Result<U, B>;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map(f)
        })
    }
}
