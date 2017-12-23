use std::sync::Arc;

use super::{Poll, Task, TaskContext};


pub fn map_err<T, F, R>(task: T, f: Arc<F>) -> MapErr<T, F>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    MapErr { task, f: Some(f) }
}


#[derive(Debug)]
pub struct MapErr<T, F> {
    task: T,
    f: Option<Arc<F>>,
}

impl<T, F, R> Task for MapErr<T, F>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    type Item = T::Item;
    type Error = R;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        match self.task.poll(ctx) {
            Ok(async) => Ok(async),
            Err(e) => {
                let f = self.f.take().expect("cannot reject twice");
                Err((*f)(e))
            }
        }
    }
}
