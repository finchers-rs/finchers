use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{Poll, Task};


/// Equivalent to `e.map(f)`
pub fn map<E, F, R>(endpoint: E, f: F) -> Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    Map {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `map(e, f)`
#[derive(Debug)]
pub struct Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;
    type Task = MapTask<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(MapTask {
            inner,
            f: self.f.clone(),
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct MapTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    inner: E::Task,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Task for MapTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.inner.poll(ctx));
        Ok((*self.f)(item).into())
    }
}
