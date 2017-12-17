use std::sync::Arc;

use context::Context;
use super::{Poll, Task};
use super::oneshot_fn::*;


pub fn map_err<T, F, R>(task: T, f: F) -> MapErr<T, F, fn(T::Error) -> R>
where
    T: Task,
    F: FnOnce(T::Error) -> R,
{
    MapErr {
        task,
        f: Some(owned(f)),
    }
}

pub fn map_err_shared<T, F, R>(task: T, f: Arc<F>) -> MapErr<T, fn(T::Error) -> R, F>
where
    T: Task,
    F: Fn(T::Error) -> R,
{
    MapErr {
        task,
        f: Some(shared(f)),
    }
}


#[derive(Debug)]
pub struct MapErr<T, F1, F2> {
    task: T,
    f: Option<OneshotFn<F1, F2>>,
}

impl<T, F1, F2, R> Task for MapErr<T, F1, F2>
where
    T: Task,
    F1: FnOnce(T::Error) -> R,
    F2: Fn(T::Error) -> R,
{
    type Item = T::Item;
    type Error = R;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        match self.task.poll(ctx) {
            Ok(async) => Ok(async),
            Err(e) => {
                let f = self.f.take().expect("cannot reject twice");
                Err(f.call(e))
            }
        }
    }
}
