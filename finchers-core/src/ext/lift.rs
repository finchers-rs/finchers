use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::poll::Poll;
use crate::task::Task;

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

impl<E> EndpointBase for Lift<E>
where
    E: EndpointBase,
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

    fn poll_task(&mut self) -> Poll<Self::Output> {
        match self.task {
            Some(ref mut t) => t.poll_task().map(Some),
            None => Poll::Ready(None),
        }
    }
}
