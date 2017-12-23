use super::*;
use futures::future::FutureResult;


pub trait Task {
    type Item;
    type Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error>;
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
    type Task = TaskFuture<FutureResult<T, E>>;

    fn into_task(self) -> Self::Task {
        from_future(self)
    }
}
