use super::chain::Chain;
use finchers_core::error::HttpError;
use finchers_core::{Caller, Error, Input};
use futures::{Future, IntoFuture, Poll};
use {Context, Endpoint};

pub fn new<E, F, R>(endpoint: E, f: F) -> TryAbort<E, F>
where
    E: Endpoint,
    F: Caller<E::Item, Output = R> + Clone,
    R: IntoFuture,
    R::Error: HttpError,
{
    TryAbort { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct TryAbort<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for TryAbort<E, F>
where
    E: Endpoint,
    F: Caller<E::Item, Output = R> + Clone,
    R: IntoFuture,
    R::Error: HttpError,
{
    type Item = R::Item;
    type Future = TryAbortFuture<E::Future, F, R>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(input, ctx)?;
        Some(TryAbortFuture {
            inner: Chain::new(future, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct TryAbortFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Caller<T::Item, Output = R>,
    R: IntoFuture,
    R::Error: HttpError,
{
    inner: Chain<T, R::Future, F>,
}

impl<T, F, R> Future for TryAbortFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Caller<T::Item, Output = R>,
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
