use std::sync::Arc;

use context::Context;
use super::{IntoTask, Poll, Task};
use super::chain::Chain;


pub fn or_else<T, A, F, R>(task: T, f: A) -> OrElse<T, F, R>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    OrElse {
        inner: Chain::new(task, f.into()),
    }
}


#[derive(Debug)]
pub struct OrElse<T, F, R>
where
    T: Task,
    F: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    inner: Chain<T, R::Task, Arc<F>>,
}

impl<T, F, R> Task for OrElse<T, F, R>
where
    T: Task,
    F: Fn(T::Error) -> R,
    R: IntoTask<Item = T::Item>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err((*f)(err).into_task())),
        })
    }
}
