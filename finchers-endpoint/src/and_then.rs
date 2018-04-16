use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, Async, PollTask, Task};
use futures::{Future, IntoFuture};
use std::mem;

pub fn new<E, F, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    AndThen { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Item = R::Item;
    type Task = AndThenTask<E::Task, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(AndThenTask::First(task, self.f.clone()))
    }
}

#[derive(Debug)]
pub enum AndThenTask<T, F, R>
where
    T: Task,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    First(T, F),
    Second(R::Future),
    Done,
}

impl<T, F, R> Task for AndThenTask<T, F, R>
where
    T: Task,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Output = R::Item;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        use self::AndThenTask::*;
        loop {
            // TODO: optimize
            match mem::replace(self, Done) {
                First(mut task, f) => match task.poll_task(cx)? {
                    Async::NotReady => {
                        *self = First(task, f);
                        return Ok(Async::NotReady);
                    }
                    Async::Ready(r) => {
                        cx.input().enter_scope(|| {
                            *self = Second(f(r).into_future());
                        });
                        continue;
                    }
                },
                Second(mut fut) => match fut.poll()? {
                    Async::NotReady => {
                        *self = Second(fut);
                        return Ok(Async::NotReady);
                    }
                    Async::Ready(item) => return Ok(Async::Ready(item)),
                },
                Done => panic!(),
            }
        }
    }
}
