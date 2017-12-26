#![allow(missing_docs)]

use std::sync::Arc;
use futures::{Future, Poll};
use super::{Task, TaskContext};

#[derive(Debug)]
pub struct Inspect<T, F> {
    pub(crate) task: T,
    pub(crate) f: Arc<F>,
}

impl<T, F> Task for Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    type Item = T::Item;
    type Error = T::Error;
    type Future = InspectFuture<T::Future, F>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let Inspect { task, f } = self;
        let fut = task.launch(ctx);
        InspectFuture { fut, f: Some(f) }
    }
}

#[derive(Debug)]
pub struct InspectFuture<T, F> {
    fut: T,
    f: Option<Arc<F>>,
}

impl<T, F> Future for InspectFuture<T, F>
where
    T: Future,
    F: Fn(&T::Item),
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        let f = self.f.take().expect("cannot resolve twice");
        (*f)(&item);
        Ok(item.into())
    }
}
