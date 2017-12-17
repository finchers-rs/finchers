use std::sync::Arc;

use context::Context;
use super::{IntoTask, Poll, Task};
use super::chain::Chain;
use super::oneshot_fn::*;


pub fn then<T, F, R>(task: T, f: F) -> Then<T, F, fn(Result<T::Item, T::Error>) -> R, R>
where
    T: Task,
    F: FnOnce(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    Then {
        inner: Chain::new(task, owned(f)),
    }
}

pub fn then_shared<T, F, R>(task: T, f: Arc<F>) -> Then<T, fn(Result<T::Item, T::Error>) -> R, F, R>
where
    T: Task,
    F: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    Then {
        inner: Chain::new(task, shared(f)),
    }
}

#[derive(Debug)]
pub struct Then<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(Result<T::Item, T::Error>) -> R,
    F2: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    inner: Chain<T, R::Task, OneshotFn<F1, F2>>,
}

impl<T, F1, F2, R> Task for Then<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(Result<T::Item, T::Error>) -> R,
    F2: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner
            .poll(ctx, |result, f| Ok(Err(f.call(result).into_task())))
    }
}
