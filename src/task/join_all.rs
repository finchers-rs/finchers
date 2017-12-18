#![allow(missing_docs)]

use std::fmt;
use super::{Async, IntoTask, Poll, Task, TaskContext};
use super::maybe_done::MaybeDone;

pub fn join_all<I>(iter: I) -> JoinAll<<I::Item as IntoTask>::Task>
where
    I: IntoIterator,
    I::Item: IntoTask,
{
    JoinAll {
        inner: iter.into_iter()
            .map(|t| MaybeDone::NotYet(t.into_task()))
            .collect(),
    }
}


pub struct JoinAll<T: Task> {
    inner: Vec<MaybeDone<T>>,
}

impl<T> fmt::Debug for JoinAll<T>
where
    T: Task + fmt::Debug,
    T::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JoinAll")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Task> Task for JoinAll<T> {
    type Item = Vec<T::Item>;
    type Error = T::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        let mut all_done = Ok(true);
        for t in &mut self.inner {
            match t.poll(ctx) {
                Ok(v) => if let Ok(ref mut all_done) = all_done {
                    *all_done = *all_done && v
                },
                Err(err) => {
                    all_done = Err(err);
                    break;
                }
            }
        }

        match all_done {
            Ok(true) => {
                let result = self.inner.iter_mut().map(|t| t.take()).collect();
                Ok(Async::Ready(result))
            }
            Ok(false) => Ok(Async::NotReady),
            Err(err) => Err(err),
        }
    }
}
