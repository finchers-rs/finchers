use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};
use http::HttpError;
use super::{Task, TaskContext};
use super::chain::Chain;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Then<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F, R> Task for Then<T, F>
where
    T: Task,
    F: Fn(Result<T::Item, T::Error>) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = ThenFuture<T::Future, F, T::Error, R>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let Then { task, f } = self;
        let fut = task.launch(ctx);
        ThenFuture {
            inner: Chain::new(fut, f),
        }
    }
}

#[derive(Debug)]
pub struct ThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(Result<T::Item, E>) -> R,
    R: IntoFuture,
{
    inner: Chain<T, R::Future, Arc<F>>,
}

impl<T, F, E, R> Future for ThenFuture<T, F, E, R>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(Result<T::Item, E>) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = Result<R::Error, HttpError>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner
            .poll(|result, f| Ok(Err((*f)(result).into_future())))
    }
}
