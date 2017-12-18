use futures::Future;
use task::{Poll, Task, TaskContext};

pub fn future<F: Future>(f: F) -> TaskFuture<F> {
    TaskFuture { inner: f }
}


#[derive(Debug)]
pub struct TaskFuture<F: Future> {
    inner: F,
}

impl<F: Future> From<F> for TaskFuture<F> {
    fn from(f: F) -> Self {
        future(f)
    }
}

impl<F: Future> Task for TaskFuture<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self, _: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
