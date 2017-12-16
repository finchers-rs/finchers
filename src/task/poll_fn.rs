#![allow(missing_docs)]

use std::marker::PhantomData;
use context::Context;
use super::{Poll, Task};

pub fn poll_fn<F, T, E>(f: F) -> PollFn<F, T, E>
where
    F: FnMut(&mut Context) -> Poll<T, E>,
{
    PollFn {
        f,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct PollFn<F, T, E>
where
    F: FnMut(&mut Context) -> Poll<T, E>,
{
    f: F,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<F, T, E> Task for PollFn<F, T, E>
where
    F: FnMut(&mut Context) -> Poll<T, E>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        (self.f)(ctx)
    }
}
