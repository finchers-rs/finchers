use std::sync::Arc;

use super::{IntoTask, Poll, Task, TaskContext};
use super::chain::Chain;
use super::oneshot_fn::*;


pub fn or_else<T, F, R>(task: T, f: F) -> OrElse<T, F, fn(T::Error) -> R, R>
where
    T: Task,
    F: FnOnce(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    OrElse {
        inner: Chain::new(task, owned(f)),
    }
}

pub fn or_else_shared<T, F, R>(task: T, f: Arc<F>) -> OrElse<T, fn(T::Error) -> R, F, R>
where
    T: Task,
    F: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    OrElse {
        inner: Chain::new(task, shared(f)),
    }
}

#[derive(Debug)]
pub struct OrElse<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(T::Error) -> R,
    F2: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    inner: Chain<T, R::Task, OneshotFn<F1, F2>>,
}

impl<T, F1, F2, R> Task for OrElse<T, F1, F2, R>
where
    T: Task,
    F1: FnOnce(T::Error) -> R,
    F2: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err(f.call(err).into_task())),
        })
    }
}
