#![allow(missing_docs)]

use futures::{Future, IntoFuture};
use task::{Poll, Task, TaskContext};

pub fn from_future<F: IntoFuture>(f: F) -> TaskFuture<F::Future> {
    TaskFuture {
        inner: f.into_future(),
    }
}


#[derive(Debug)]
pub struct TaskFuture<F: Future> {
    inner: F,
}

impl<F: Future> From<F> for TaskFuture<F> {
    fn from(f: F) -> Self {
        from_future(f)
    }
}

impl<F: Future> Task for TaskFuture<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self, _: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
