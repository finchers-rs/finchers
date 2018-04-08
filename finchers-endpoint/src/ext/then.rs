use super::chain::Chain;
use callable::Callable;
use finchers_core::Input;
use futures::{Future, IntoFuture, Poll};
use {Context, Endpoint, Error};

pub fn new<E, F, R>(endpoint: E, f: F) -> Then<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = R> + Clone,
    R: IntoFuture<Error = !>,
{
    Then { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct Then<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for Then<E, F>
where
    E: Endpoint,
    F: Callable<E::Item, Output = R> + Clone,
    R: IntoFuture<Error = !>,
{
    type Item = R::Item;
    type Future = ThenFuture<E::Future, F, R>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(input, ctx)?;
        Some(ThenFuture {
            inner: Chain::new(future, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Callable<T::Item, Output = R>,
    R: IntoFuture<Error = !>,
{
    inner: Chain<T, R::Future, F>,
}

impl<T, F, R> Future for ThenFuture<T, F, R>
where
    T: Future<Error = Error>,
    F: Callable<T::Item, Output = R>,
    R: IntoFuture<Error = !>,
{
    type Item = R::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err(f.call(item).into_future())),
            Err(..) => unreachable!(),
        })
    }
}
