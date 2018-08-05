#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::task::Task;
use finchers_core::{Error, PollResult};

pub fn new<E, F>(endpoint: E, f: F) -> Inspect<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(&E::Output) + Clone + Send + Sync,
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

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: FnOnce(&E::Output) + Clone + Send + Sync,
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
    T: Task + Send,
    F: FnOnce(&T::Output) + Send,
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
