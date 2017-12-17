use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use super::{Poll, Task};


pub fn map_err<T, A, F, R>(task: T, f: A) -> MapErr<T, F, R>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(T::Error) -> R,
{
    MapErr {
        task,
        f: f.into(),
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct MapErr<T, F, R>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    task: T,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<T, F, R> Task for MapErr<T, F, R>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    type Item = T::Item;
    type Error = R;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.task.poll(ctx).map_err(|e| (*self.f)(e))
    }
}
