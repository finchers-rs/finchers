use std::sync::Arc;

use futures::{Future, Poll};
use http::HttpError;
use super::{Task, TaskContext};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct MapErr<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F, R> Task for MapErr<T, F>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    type Item = T::Item;
    type Error = R;
    type Future = MapErrFuture<T::Future, F>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let MapErr { task, f } = self;
        let fut = task.launch(ctx);
        MapErrFuture { fut, f: Some(f) }
    }
}

#[derive(Debug)]
pub struct MapErrFuture<T, F> {
    fut: T,
    f: Option<Arc<F>>,
}

impl<T, F, E, R> Future for MapErrFuture<T, F>
where
    T: Future<Error = Result<E, HttpError>>,
    F: Fn(E) -> R,
{
    type Item = T::Item;
    type Error = Result<R, HttpError>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.fut.poll() {
            Ok(async) => Ok(async),
            Err(e) => {
                let f = self.f.take().expect("cannot reject twice");
                Err(e.map(|e| (*f)(e)))
            }
        }
    }
}
