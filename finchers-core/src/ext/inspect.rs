#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::task::Task;
use crate::{Error, PollResult};

pub fn new<E, F>(endpoint: E, f: F) -> Inspect<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(&E::Output) + Clone,
{
    Inspect {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Inspect<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F> EndpointBase for Inspect<E, F>
where
    E: EndpointBase,
    F: FnOnce(&E::Output) + Clone,
{
    type Output = E::Output;
    type Task = InspectTask<E::Task, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(InspectTask {
            task: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct InspectTask<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F> Task for InspectTask<T, F>
where
    T: Task,
    F: FnOnce(&T::Output),
{
    type Output = T::Output;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        self.task.poll_task().map_ok(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            f(&item);
            item
        })
    }
}
