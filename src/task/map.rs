use std::sync::Arc;

use super::{Poll, Task, TaskContext};


pub fn map<T, F, R>(task: T, f: Arc<F>) -> Map<T, F>
where
    T: Task,
    F: FnOnce(T::Item) -> R,
{
    Map { task, f: Some(f) }
}


#[derive(Debug)]
pub struct Map<T, F> {
    task: T,
    f: Option<Arc<F>>,
}

impl<T, F, R> Task for Map<T, F>
where
    T: Task,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        let f = self.f.take().expect("cannot resolve twice");
        Ok((*f)(item).into())
    }
}
