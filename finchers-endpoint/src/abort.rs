use finchers_core::endpoint::{Context, Endpoint, Error, IntoEndpoint};
use finchers_core::{HttpError, Input};
use futures::{Future, Poll};

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

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(input, ctx)?;
        Some(AbortFuture { fut })
    }
}

#[derive(Debug)]
pub struct AbortFuture<T> {
    fut: T,
}

impl<T> Future for AbortFuture<T>
where
    T: Future<Error = Error>,
    T::Item: HttpError,
{
    type Item = !;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        Err(Error::from(item).into())
    }
}
