use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};

use super::{Task, TaskContext};
use super::chain::Chain;


#[derive(Debug)]
pub struct AndThen<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F, R> Task for AndThen<T, F>
where
    T: Task,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = T::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = AndThenFuture<T::Future, F, R>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let AndThen { task, f } = self;
        let fut = task.launch(ctx);
        AndThenFuture {
            inner: Chain::new(fut, f),
        }
    }
}

#[derive(Debug)]
pub struct AndThenFuture<T, F, R>
where
    T: Future,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = T::Error>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}


impl<T, F, R> Future for AndThenFuture<T, F, R>
where
    T: Future,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = T::Error>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_future())),
            Err(err) => Err(err),
        })
    }
}
