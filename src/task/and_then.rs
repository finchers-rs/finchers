use std::sync::Arc;

use context::Context;
use super::{IntoTask, Poll, Task};
use super::chain::Chain;


pub fn and_then<T, A, F, R>(task: T, f: A) -> AndThen<T, F, R>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    AndThen {
        inner: Chain::new(task, f.into()),
    }
}


#[derive(Debug)]
pub struct AndThen<T, F, R>
where
    T: Task,
    F: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    inner: Chain<T, R::Task, Arc<F>>,
}

impl<T, F, R> Task for AndThen<T, F, R>
where
    T: Task,
    F: Fn(T::Item) -> R,
    R: IntoTask<Error = T::Error>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_task())),
            Err(err) => Err(err),
        })
    }
}
