use std::marker::PhantomData;

use super::{Poll, Task, TaskContext};


pub fn from_err<T, E>(task: T) -> FromErr<T, E>
where
    T: Task,
    E: From<T::Error>,
{
    FromErr {
        task,
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct FromErr<T, E>
where
    T: Task,
    E: From<T::Error>,
{
    task: T,
    _marker: PhantomData<E>,
}

impl<T, E> Task for FromErr<T, E>
where
    T: Task,
    E: From<T::Error>,
{
    type Item = T::Item;
    type Error = E;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        self.task.poll(ctx).map_err(Into::into)
    }
}
