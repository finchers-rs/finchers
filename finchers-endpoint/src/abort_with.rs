use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint, task::{self, Future, Poll}};

pub fn new<E, F, U>(endpoint: E, f: F) -> AbortWith<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Item) -> U + Clone + Send,
    U: HttpError,
{
    AbortWith {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbortWith<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, U> Endpoint for AbortWith<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> U + Clone + Send,
    U: HttpError,
{
    type Item = !;
    type Future = AbortWithFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(cx)?;
        Some(AbortWithFuture {
            fut,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AbortWithFuture<T, F> {
    fut: T,
    f: Option<F>,
}

impl<T, F, U> Future for AbortWithFuture<T, F>
where
    T: Future + Send,
    F: FnOnce(T::Item) -> U + Send,
    U: HttpError,
{
    type Item = !;

    fn poll(&mut self, cx: &mut task::Context) -> Poll<Self::Item> {
        let item = try_ready!(self.fut.poll(cx));
        let f = self.f.take().expect("cannot resolve twice");
        Err(f(item).into())
    }
}
