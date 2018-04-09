use callable::Callable;
use finchers_core::Input;
use futures::{Future, Poll};
use {Context, Endpoint, IntoEndpoint};

pub fn new<E, F>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: Callable<E::Item> + Clone,
{
    Map {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Map<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: Callable<E::Item> + Clone,
{
    type Item = F::Output;
    type Future = MapFuture<E::Future, F>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(input, ctx)?;
        Some(MapFuture {
            fut,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    fut: T,
    f: Option<F>,
}

impl<T, F> Future for MapFuture<T, F>
where
    T: Future,
    F: Callable<T::Item>,
{
    type Item = F::Output;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        let f = self.f.take().expect("cannot resolve twice");
        Ok(f.call(item).into())
    }
}
