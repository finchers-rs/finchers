use callable::Callable;
use finchers_core::{HttpError, Input};
use futures::{Future, Poll};
use {Context, Endpoint, Error, IntoEndpoint};

pub fn new<E, F>(endpoint: E, f: F) -> AbortWith<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: Callable<E::Item> + Clone,
    F::Output: HttpError,
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

impl<E, F> Endpoint for AbortWith<E, F>
where
    E: Endpoint,
    F: Callable<E::Item> + Clone,
    F::Output: HttpError,
{
    type Item = !;
    type Future = AbortWithFuture<E::Future, F>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let fut = self.endpoint.apply(input, ctx)?;
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

impl<T, F> Future for AbortWithFuture<T, F>
where
    T: Future<Error = Error>,
    F: Callable<T::Item>,
    F::Output: HttpError,
{
    type Item = !;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.fut.poll());
        let f = self.f.take().expect("cannot resolve twice");
        Err(f.call(item).into())
    }
}
