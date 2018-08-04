#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, PollResult};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

pub fn new<E, U, A, B>(endpoint: E) -> ErrInto<E, U>
where
    E: Endpoint<Output = Result<A, B>>,
    B: Into<U>,
{
    ErrInto {
        endpoint,
        _marker: PhantomData,
    }
}

impl<E, A, B, U> Endpoint for ErrInto<E, U>
where
    E: Endpoint<Output = Result<A, B>>,
    B: Into<U>,
{
    type Output = Result<A, U>;
    type Task = ErrIntoTask<E::Task, U>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(ErrIntoTask {
            task: self.endpoint.apply(cx)?,
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

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        self.task
            .poll_task(cx)
            .map_ok(|item| item.map_err(Into::into))
    }
}
