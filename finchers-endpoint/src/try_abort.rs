use callable::Callable;
use finchers_core::endpoint::{Context, Endpoint, Error};
use finchers_core::{HttpError, Input};
use futures::{Future, Poll};

pub fn new<E, F, T, R>(endpoint: E, f: F) -> TryAbort<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = Result<T, R>> + Clone,
    R: HttpError,
{
    TryAbort { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct TryAbort<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, T, R> Endpoint for TryAbort<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = Result<T, R>> + Clone,
    R: HttpError,
{
    type Item = T;
    type Future = TryAbortFuture<E::Future, F>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(input, ctx)?;
        Some(TryAbortFuture {
            future,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct TryAbortFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, U, E> Future for TryAbortFuture<T, F>
where
    T: Future<Error = Error>,
    F: Callable<T::Item, Output = Result<U, E>> + Clone,
    E: HttpError,
{
    type Item = U;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.future.poll());
        let f = self.f.take().expect("cannot resolve/reject twice");
        f.call(item).map_err(Into::into).map(Into::into)
    }
}
