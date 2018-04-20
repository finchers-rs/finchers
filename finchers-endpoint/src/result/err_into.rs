use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, PollTask, Task};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

pub fn new<E, U, A, B>(endpoint: E) -> ErrInto<E, U>
where
    E: Endpoint<Item = Result<A, B>>,
    B: Into<U>,
{
    ErrInto {
        endpoint,
        _marker: PhantomData,
    }
}

impl<E, A, B, U> Endpoint for ErrInto<E, U>
where
    E: Endpoint<Item = Result<A, B>>,
    B: Into<U>,
{
    type Item = Result<A, U>;
    type Task = ErrIntoTask<E::Task, U>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(ErrIntoTask {
            task,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct ErrIntoTask<T, U> {
    task: T,
    _marker: PhantomData<fn() -> U>,
}

impl<T, U, A, B> Task for ErrIntoTask<T, U>
where
    T: Task<Output = Result<A, B>> + Send,
    B: Into<U>,
{
    type Output = Result<A, U>;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        let item = try_ready!(self.task.poll_task(cx));
        cx.input().enter_scope(|| Ok(item.map_err(Into::into).into()))
    }
}
