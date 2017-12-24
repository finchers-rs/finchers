#![allow(missing_docs)]

use super::{Task, TaskContext};

pub fn lazy<F, R>(f: F) -> Lazy<F>
where
    F: FnOnce(&mut TaskContext) -> R,
    R: Task,
{
    Lazy {
        f,
    }
}

#[derive(Debug)]
pub struct Lazy<F> {
    f: F,
}

impl<F, R> Task for Lazy<F>
where
    F: FnOnce(&mut TaskContext) -> R,
    R: Task,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = R::Future;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        self.f.launch(ctx).launch(ctx)
    }
}