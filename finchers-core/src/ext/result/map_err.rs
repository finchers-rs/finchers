#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, PollResult};

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapErr<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> U + Clone,
{
    MapErr { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for MapErr<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> U + Clone,
{
    type Output = Result<A, U>;
    type Task = MapErrTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(MapErrTask {
            task: self.endpoint.apply(cx)?,
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
    T: Task<Output = Result<A, B>>,
    F: FnOnce(B) -> U,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map_err(f)
        })
    }
}
