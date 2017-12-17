use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use super::{Poll, Task};


pub fn map<T, A, F, R>(task: T, f: A) -> Map<T, F, R>
where
    T: Task,
    A: Into<Arc<F>>,
    F: Fn(T::Item) -> R,
{
    Map {
        task,
        f: f.into(),
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct Map<T, F, R>
where
    T: Task,
    F: Fn(T::Item) -> R,
{
    task: T,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<T, F, R> Task for Map<T, F, R>
where
    T: Task,
    F: Fn(T::Item) -> R,
{
    type Item = R;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.task.poll(ctx));
        Ok((*self.f)(item).into())
    }
}
