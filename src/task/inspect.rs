#![allow(missing_docs)]

use std::sync::Arc;

use super::{Poll, Task, TaskContext};


pub fn inspect<T, F>(task: T, f: Arc<F>) -> Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    Inspect { task, f: Some(f) }
}


#[derive(Debug)]
pub struct Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    task: T,
    f: Option<Arc<F>>,
}

impl<T, F> Task for Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        let f = self.f.take().expect("cannot resolve twice");
        (*f)(&item);
        Ok(item.into())
    }
}
