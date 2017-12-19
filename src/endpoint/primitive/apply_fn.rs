use std::marker::PhantomData;
use std::sync::Arc;
use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::IntoTask;

pub fn apply_fn<F, T>(f: F) -> ApplyFn<F, T>
where
    F: Fn(&mut EndpointContext) -> Result<T, EndpointError>,
    T: IntoTask,
{
    ApplyFn {
        f: Arc::new(f),
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct ApplyFn<F, T>
where
    F: Fn(&mut EndpointContext) -> Result<T, EndpointError>,
    T: IntoTask,
{
    f: Arc<F>,
    _marker: PhantomData<fn() -> T::Task>,
}

impl<F, T> Endpoint for ApplyFn<F, T>
where
    F: Fn(&mut EndpointContext) -> Result<T, EndpointError>,
    T: IntoTask,
{
    type Item = T::Item;
    type Error = T::Error;
    type Task = T::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        (*self.f)(ctx).map(IntoTask::into_task)
    }
}
