use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use super::{Task, TaskContext};
use super::chain::Chain;

#[derive(Debug)]
pub struct OrElse<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F, R> Task for OrElse<T, F>
where
    T: Task,
    F: Fn(T::Error) -> R,
    R: IntoFuture<Item = T::Item>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = OrElseFuture<T::Future, F, R>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let OrElse { task, f } = self;
        let fut = task.launch(ctx);
        OrElseFuture {
            inner: Chain::new(fut, f),
        }
    }
}

#[derive(Debug)]
pub struct OrElseFuture<T, F, R>
where
    T: Future,
    F: Fn(T::Error) -> R,
    R: IntoFuture<Item = T::Item>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, R> Future for OrElseFuture<T, F, R>
where
    T: Future,
    F: Fn(T::Error) -> R,
    R: IntoFuture<Item = T::Item>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err((*f)(err).into_future())),
        })
    }
}
