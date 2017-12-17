use std::sync::Arc;

use context::Context;
use super::{IntoTask, Poll, Task};
use super::chain::Chain;


pub fn then<T, A, F, R>(task: T, f: A) -> Then<T, F, R>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    Then {
        inner: Chain::new(task, f.into()),
    }
}


#[derive(Debug)]
pub struct Then<T, F, R>
where
    T: Task,
    F: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    inner: Chain<T, R::Task, Arc<F>>,
}

impl<T, F, R> Task for Then<T, F, R>
where
    T: Task,
    F: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner
            .poll(ctx, |result, f| Ok(Err((*f)(result).into_task())))
    }
}
