#![allow(missing_docs)]

mod result;
mod futures;

use context::Context;

pub use futures::{Async, Poll};
pub use self::result::{err, ok, result, TaskResult};
pub use self::futures::{future, TaskFuture};

pub trait Task {
    type Item;
    type Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error>;
}


pub trait IntoTask {
    type Item;
    type Error;
    type Task: Task<Item = Self::Item, Error = Self::Error>;

    fn into_task(self) -> Self::Task;
}

impl<T: Task> IntoTask for T {
    type Item = T::Item;
    type Error = T::Error;
    type Task = T;
    fn into_task(self) -> Self::Task {
        self
    }
}

impl<T, E> IntoTask for Result<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskResult<T, E>;

    fn into_task(self) -> Self::Task {
        result(self)
    }
}
