use std::sync::Arc;

use context::Context;
use super::{IntoTask, Poll, Task};
use super::chain::Chain;
use super::oneshot_fn::{owned, shared, Caller, OneshotFn};


pub fn and_then<T, F, R>(task: T, f: F) -> AndThen<T, F, fn(T::Item) -> R, R>
where
    T: Task,
    F: FnOnce(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    AndThen {
        inner: Chain::new(task, owned(f)),
    }
}

pub fn and_then_shared<T, F, R>(task: T, f: Arc<F>) -> AndThen<T, fn(T::Item) -> R, F, R>
where
    T: Task,
    F: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    AndThen {
        inner: Chain::new(task, shared(f)),
    }
}


#[derive(Debug)]
pub struct AndThen<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(T::Item) -> R,
    F2: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    inner: Chain<T, R::Task, OneshotFn<F1, F2>>,
}

impl<T, F1, F2, R> Task for AndThen<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(T::Item) -> R,
    F2: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Err(f.call(item).into_task())),
            Err(err) => Err(err),
        })
    }
}
