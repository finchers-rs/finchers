#![allow(missing_docs)]

use task::{Async, Poll, Task, TaskContext};


pub fn ok<T, E>(x: T) -> TaskResult<T, E> {
    TaskResult { inner: Some(Ok(x)) }
}

pub fn err<T, E>(e: E) -> TaskResult<T, E> {
    TaskResult {
        inner: Some(Err(e)),
    }
}

pub fn result<T, E>(res: Result<T, E>) -> TaskResult<T, E> {
    TaskResult { inner: Some(res) }
}

#[derive(Debug)]
pub struct TaskResult<T, E> {
    inner: Option<Result<T, E>>,
}

impl<T, E> From<Result<T, E>> for TaskResult<T, E> {
    fn from(res: Result<T, E>) -> Self {
        result(res)
    }
}

impl<T, E> Task for TaskResult<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self, _: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        self.inner
            .take()
            .expect("cannot resolve twice")
            .map(Async::Ready)
    }
}
