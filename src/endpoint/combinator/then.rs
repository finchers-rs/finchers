use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};
use super::chain::Chain;


/// Equivalent to `e.then(f)`
pub fn then<E, F, R>(endpoint: E, f: F) -> Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    Then {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `then()`
#[derive(Debug)]
pub struct Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = ThenFuture<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let fut = self.endpoint.apply(ctx)?;
        Ok(ThenFuture {
            inner: Chain::new(fut, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ThenFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    inner: Chain<E::Future, R::Future, Arc<F>>,
}

impl<E, F, R> Future for ThenFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner
            .poll(|result, f| Ok(Err((*f)(result).into_future())))
    }
}
