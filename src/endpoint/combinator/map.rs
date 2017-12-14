use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};


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
    type Future = MapFuture<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(MapFuture {
            inner,
            f: self.f.clone(),
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct MapFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    inner: E::Future,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Future for MapFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.inner.poll());
        Ok((*self.f)(item).into())
    }
}
