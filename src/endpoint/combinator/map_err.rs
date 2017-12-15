use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{Poll, Task};


/// Equivalent to `e.map_err(f)`
pub fn map_err<E, F, R>(endpoint: E, f: F) -> MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    MapErr {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `map_err(e, f)`
#[derive(Debug)]
pub struct MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Task = MapErrTask<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(MapErrTask {
            inner,
            f: self.f.clone(),
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct MapErrTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    inner: E::Task,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Task for MapErrTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx).map_err(|e| (*self.f)(e))
    }
}
