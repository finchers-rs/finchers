#![allow(missing_docs)]

use std::sync::Arc;

use super::{Poll, Task, TaskContext};
use super::oneshot_fn::*;


pub fn inspect<T, F>(task: T, f: F) -> Inspect<T, F, fn(&T::Item)>
where
    T: Task,
    F: FnOnce(&T::Item),
{
    Inspect {
        task,
        f: Some(owned(f)),
    }
}

pub fn inspect_shared<T, F>(task: T, f: Arc<F>) -> Inspect<T, fn(&T::Item), F>
where
    T: Task,
    F: Fn(&T::Item),
{
    Inspect {
        task,
        f: Some(shared(f)),
    }
}


#[derive(Debug)]
pub struct Inspect<T, F1, F2>
where
    T: Task,
    F1: FnOnce(&T::Item),
    F2: Fn(&T::Item),
{
    task: T,
    f: Option<OneshotFn<F1, F2>>,
}

impl<T, F1, F2> Task for Inspect<T, F1, F2>
where
    T: Task,
    F1: FnOnce(&T::Item),
    F2: Fn(&T::Item),
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        self.f.take().expect("cannot resolve twice").call(&item);
        Ok(item.into())
    }
}
