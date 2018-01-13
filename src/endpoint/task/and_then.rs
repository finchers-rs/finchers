use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use http::{HttpError, Request};
use super::Task;
use super::chain::Chain;

#[allow(missing_docs)]
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
    type Future = AndThenFuture<T::Future, F, T::Error, R>;

    fn launch(self, request: &mut Request) -> Self::Future {
        let AndThen { task, f } = self;
        let fut = task.launch(request);
        AndThenFuture {
            inner: Chain::new(fut, f),
        }
    }
}

#[derive(Debug)]
pub struct AndThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = E>,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, E, R> Future for AndThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(T::Item) -> R,
    R: IntoFuture<Error = E>,
{
    type Item = R::Item;
    type Error = Result<R::Error, HttpError>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_future())),
            Err(err) => Err(err),
        })
    }
}
