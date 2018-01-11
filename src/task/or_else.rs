use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use http::HttpError;
use super::{Task, TaskContext};
use super::chain::Chain;

#[allow(missing_docs)]
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
    type Future = OrElseFuture<T::Future, F, T::Error, R>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let OrElse { task, f } = self;
        let fut = task.launch(ctx);
        OrElseFuture {
            inner: Chain::new(fut, f),
        }
    }
}

#[derive(Debug)]
pub struct OrElseFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(E) -> R,
    R: IntoFuture<Item = T::Item>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, E, R> Future for OrElseFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(E) -> R,
    R: IntoFuture<Item = T::Item>,
{
    type Item = R::Item;
    type Error = Result<R::Error, HttpError>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err((*f)(err).into_future())),
        })
    }
}
