#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::task::Task;
use crate::{Error, Poll, PollResult};

pub fn new<E, T, R>(endpoint: E) -> UnwrapOk<E>
where
    E: EndpointBase<Output = Result<T, R>>,
    R: Into<Error>,
{
    UnwrapOk { endpoint }
}

#[derive(Copy, Clone, Debug)]
pub struct UnwrapOk<E> {
    endpoint: E,
}

impl<E, T, R> EndpointBase for UnwrapOk<E>
where
    E: EndpointBase<Output = Result<T, R>>,
    R: Into<Error>,
{
    type Output = T;
    type Task = UnwrapOkTask<E::Task>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(UnwrapOkTask {
            task: self.endpoint.apply(cx)?,
        })
    }
}

#[derive(Debug)]
pub struct UnwrapOkTask<T> {
    task: T,
}

impl<T, U, E> Task for UnwrapOkTask<T>
where
    T: Task<Output = Result<U, E>>,
    E: Into<Error>,
{
    type Output = U;

    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        let res: Result<U, E> = poll_result!(self.task.poll_task());
        Poll::Ready(res.map_err(Into::into))
    }
}
