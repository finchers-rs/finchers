use super::chain::Chain;
use callable::Callable;
use finchers_core::endpoint::{Context, Endpoint, Error};
use finchers_core::{HttpError, Input};
use futures::{Future, IntoFuture, Poll};

pub fn new<E, F, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = R> + Clone,
    R: IntoFuture,
    R::Error: HttpError,
{
    AndThen { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = R> + Clone,
    R: IntoFuture,
    R::Error: HttpError,
{
    type Item = R::Item;
    type Future = AndThenFuture<E::Future, F, R>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(input, ctx)?;
        Some(AndThenFuture {
            inner: Chain::new(future, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AndThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Callable<T::Item, Output = R>,
    R: IntoFuture,
    R::Error: HttpError,
{
    inner: Chain<T, R::Future, F>,
}

impl<T, F, R> Future for AndThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Callable<T::Item, Output = R>,
    R: IntoFuture,
    R::Error: HttpError,
{
    type Item = R::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err(f.call(item).into_future())),
            Err(err) => Err(err),
        })
    }
}
