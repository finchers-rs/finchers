use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};
use super::chain::Chain;


/// Equivalent to `e.or_else(f)`
pub fn or_else<E, F, R>(endpoint: E, f: F) -> OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    OrElse {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `or_else()`
#[derive(Debug)]
pub struct OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

// The implementation of `Endpoint` for `AndThen`.
impl<E, F, R> Endpoint for OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = OrElseFuture<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let fut = self.endpoint.apply(ctx)?;
        Ok(OrElseFuture {
            inner: Chain::new(fut, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OrElseFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    inner: Chain<E::Future, R::Future, Arc<F>>,
}

impl<E, F, R> Future for OrElseFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err((*f)(err).into_future())),
        })
    }
}
