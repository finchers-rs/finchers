#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;
use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::IntoTask;


pub fn lazy<F, R>(f: F) -> Lazy<F, R>
where
    F: Fn() -> R,
    R: IntoTask,
{
    Lazy {
        f: Arc::new(f),
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct Lazy<F, R>
where
    F: Fn() -> R,
    R: IntoTask,
{
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<F, R> Endpoint for Lazy<F, R>
where
    F: Fn() -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = R::Task;

    fn apply(&self, _: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Ok((*self.f)().into_task())
    }
}
