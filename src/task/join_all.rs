#![allow(missing_docs)]

use futures::future;
use super::{Task, TaskContext};

#[derive(Debug)]
pub struct JoinAll<T> {
    pub(crate) inner: Vec<T>,
}

impl<T> Task for JoinAll<T>
where
    T: Task,
{
    type Item = Vec<T::Item>;
    type Error = T::Error;
    type Future = future::JoinAll<Vec<T::Future>>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        future::join_all(self.inner.into_iter().map(|t| t.launch(ctx)).collect())
    }
}
