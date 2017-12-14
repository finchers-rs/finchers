use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Future, IntoFuture, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};
use super::chain::Chain;


/// Equivalent to `e.and_then(f)`
pub fn and_then<E, F, R>(endpoint: E, f: F) -> AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    AndThen {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `and_then()`
#[derive(Debug)]
pub struct AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = AndThenFuture<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let f = self.endpoint.apply(ctx)?;
        Ok(AndThenFuture {
            inner: Chain::new(f, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AndThenFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    inner: Chain<E::Future, R::Future, Arc<F>>,
}

impl<E, F, R> Future for AndThenFuture<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(|result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_future())),
            Err(err) => Err(err),
        })
    }
}
