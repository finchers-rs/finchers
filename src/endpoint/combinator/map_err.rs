use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};


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
    type Future = MapErrFuture<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(MapErrFuture {
            inner,
            f: self.f.clone(),
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct MapErrFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    inner: E::Future,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Future for MapErrFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll().map_err(|e| (*self.f)(e))
    }
}
