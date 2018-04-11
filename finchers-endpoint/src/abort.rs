use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, Error, IntoEndpoint, task::{self, Future, Poll}};

pub fn new<E>(endpoint: E) -> Abort<E::Endpoint>
where
    E: IntoEndpoint,
    E::Item: HttpError,
{
    Abort {
        endpoint: endpoint.into_endpoint(),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Abort<E> {
    endpoint: E,
}

impl<E> Endpoint for Abort<E>
where
    E: Endpoint,
    E::Item: HttpError,
{
    type Item = !;
    type Future = AbortFuture<E::Future>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(cx)?;
        Some(AbortFuture { fut })
    }
}

#[derive(Debug)]
pub struct AbortFuture<T> {
    fut: T,
}

impl<T> Future for AbortFuture<T>
where
    T: Future,
    T::Item: HttpError,
{
    type Item = !;

    fn poll(&mut self, cx: &mut task::Context) -> Poll<Self::Item> {
        let item = try_ready!(self.fut.poll(cx));
        Err(Error::from(item).into())
    }
}
