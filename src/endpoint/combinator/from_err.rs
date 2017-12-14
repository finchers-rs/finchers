#![allow(missing_docs)]

use std::marker::PhantomData;
use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};


pub fn from_err<E, T>(endpoint: E) -> FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    FromErr {
        endpoint,
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct FromErr<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

impl<E, T> Endpoint for FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    type Item = E::Item;
    type Error = T;
    type Future = FromErrFuture<E, T>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(FromErrFuture {
            inner,
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct FromErrFuture<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    inner: E::Future,
    _marker: PhantomData<T>,
}

impl<E, T> Future for FromErrFuture<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    type Item = E::Item;
    type Error = T;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll().map_err(T::from)
    }
}
