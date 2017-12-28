use std::sync::Arc;
use futures::{Future, Poll};
use super::{Task, TaskContext};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Map<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F, R> Task for Map<T, F>
where
    T: Task,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;
    type Future = MapFuture<T::Future, F>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let Map { task, f } = self;
        let fut = task.launch(ctx);
        MapFuture { fut, f: Some(f) }
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    fut: T,
    f: Option<Arc<F>>,
}

impl<T, F, R> Future for MapFuture<T, F>
where
    T: Future,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        let f = self.f.take().expect("cannot resolve twice");
        Ok((*f)(item).into())
    }
}
