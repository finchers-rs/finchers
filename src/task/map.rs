use std::sync::Arc;

use super::{Poll, Task, TaskContext};
use super::oneshot_fn::*;


pub fn map<T, F, R>(task: T, f: F) -> Map<T, F, fn(T::Item) -> R>
where
    T: Task,
    F: FnOnce(T::Item) -> R,
{
    Map {
        task,
        f: Some(owned(f)),
    }
}

pub fn map_shared<T, F, R>(task: T, f: Arc<F>) -> Map<T, fn(T::Item) -> R, F>
where
    T: Task,
    F: FnOnce(T::Item) -> R,
{
    Map {
        task,
        f: Some(shared(f)),
    }
}


#[derive(Debug)]
pub struct Map<T, F1, F2> {
    task: T,
    f: Option<OneshotFn<F1, F2>>,
}

impl<T, F1, F2, R> Task for Map<T, F1, F2>
where
    T: Task,
    F1: FnOnce(T::Item) -> R,
    F2: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        let f = self.f.take().expect("cannot resolve twice");
        Ok(f.call(item).into())
    }
}
