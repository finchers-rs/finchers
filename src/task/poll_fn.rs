use std::marker::PhantomData;
use super::{Poll, Task, TaskContext};

pub fn poll_fn<F, T, E>(f: F) -> PollFn<F, T, E>
where
    F: FnMut(&mut TaskContext) -> Poll<T, E>,
{
    PollFn {
        f,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct PollFn<F, T, E>
where
    F: FnMut(&mut TaskContext) -> Poll<T, E>,
{
    f: F,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<F, T, E> Task for PollFn<F, T, E>
where
    F: FnMut(&mut TaskContext) -> Poll<T, E>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        (self.f)(ctx)
    }
}
