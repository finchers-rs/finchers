#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, Poll};

#[derive(Debug, Copy, Clone)]
pub struct MapSome<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, T>(endpoint: E, f: F) -> MapSome<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone,
{
    MapSome { endpoint, f }
}

impl<E, F, T, U> EndpointBase for MapSome<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone,
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
    T: Task<Output = Option<A>>,
    F: FnOnce(A) -> U,
{
    type Output = Option<U>;

    fn poll_task(&mut self) -> Poll<Result<Self::Output, Error>> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map(f)
        })
    }
}
