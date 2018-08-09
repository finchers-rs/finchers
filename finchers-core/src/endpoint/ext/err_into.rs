#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
use crate::generic::Tuple;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, U> {
    pub(super) endpoint: E,
    pub(super) _marker: PhantomData<fn() -> U>,
}

impl<E, U> EndpointBase for ErrInto<E, U>
where
    E: EndpointBase,
    E::Error: Into<U>,
{
    type Ok = E::Ok;
    type Error = U;
    type Future = ErrIntoFuture<E::Future, U>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(ErrIntoFuture {
            future: self.endpoint.apply(cx)?,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct ErrIntoFuture<T, U> {
    future: T,
    _marker: PhantomData<fn() -> U>,
}

impl<T, U> Future for ErrIntoFuture<T, U>
where
    T: TryFuture,
    T::Ok: Tuple,
    T::Error: Into<U>,
{
    type Output = Result<T::Ok, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.try_poll().map_err(Into::into)
    }
}
