use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, Poll, PollResult};

pub fn new<E>(endpoint: E) -> Lift<E::Endpoint>
where
    E: IntoEndpoint,
{
    Lift {
        endpoint: endpoint.into_endpoint(),
    }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Lift<E> {
    endpoint: E,
}

impl<E> Endpoint for Lift<E>
where
    E: Endpoint,
{
    type Output = Option<E::Output>;
    type Task = LiftTask<E::Task>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(LiftTask {
            task: self.endpoint.apply(cx),
        })
    }
}

#[derive(Debug)]
pub struct LiftTask<T> {
    task: Option<T>,
}

impl<T> Task for LiftTask<T>
where
    T: Task,
{
    type Output = Option<T::Output>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        match self.task {
            Some(ref mut t) => t.poll_task(cx).map_ok(Some),
            None => Poll::Ready(Ok(None)),
        }
    }
}
