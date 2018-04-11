use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint, task::{self, Future, Poll}};

pub fn new<E, F, T>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Item) -> T + Clone + Send,
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

impl<E, F, T> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> T + Clone + Send,
{
    type Item = F::Output;
    type Future = MapFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(cx)?;
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

impl<T, F, U> Future for MapFuture<T, F>
where
    T: Future + Send,
    F: FnOnce(T::Item) -> U + Send,
{
    type Item = U;

    fn poll(&mut self, cx: &mut task::Context) -> Poll<Self::Item> {
        let item = try_ready!(self.fut.poll(cx));
        let f = self.f.take().expect("cannot resolve twice");
        Ok(f(item).into())
    }
}
