use super::chain::Chain;
use finchers_core::endpoint::{Context, Endpoint, Error};
use futures::{Future, IntoFuture, Poll};

pub fn new<E, F, R>(endpoint: E, f: F) -> Then<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture<Error = !>,
    R::Future: Send,
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
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture<Error = !>,
    R::Future: Send,
{
    type Item = R::Item;
    type Future = ThenFuture<E::Future, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        Some(ThenFuture {
            inner: Chain::new(future, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ThenFuture<T, F, R>
where
    T: Future<Error = Error> + Send,
    F: FnOnce(T::Item) -> R + Send,
    R: IntoFuture<Error = !>,
    R::Future: Send,
{
    inner: Chain<T, R::Future, F>,
}

impl<T, F, R> Future for ThenFuture<T, F, R>
where
    T: Future<Error = Error> + Send,
    F: FnOnce(T::Item) -> R + Send,
    R: IntoFuture<Error = !>,
    R::Future: Send,
{
    type Item = R::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err(f(item).into_future())),
            Err(..) => unreachable!(),
        })
    }
}
