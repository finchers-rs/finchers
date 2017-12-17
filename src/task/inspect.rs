#![allow(missing_docs)]

use std::sync::Arc;

use context::Context;
use super::{Poll, Task};


pub fn inspect<T, F, A>(task: T, f: A) -> Inspect<T, F>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(&T::Item),
{
    Inspect { task, f: f.into() }
}


#[derive(Debug)]
pub struct Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    task: T,
    f: Arc<F>,
}

impl<T, F> Task for Inspect<T, F>
where
    T: Task,
    F: Fn(&T::Item),
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        (*self.f)(&item);
        Ok(item.into())
    }
}
