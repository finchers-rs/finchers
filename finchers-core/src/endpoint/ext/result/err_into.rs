#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{map_one, One};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, T> {
    pub(super) endpoint: E,
    pub(super) _marker: PhantomData<fn() -> T>,
}

impl<E, A, B, U> EndpointBase for ErrInto<E, U>
where
    E: EndpointBase<Output = One<Result<A, B>>>,
    B: Into<U>,
{
    type Output = One<Result<A, U>>;
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

impl<T, U, A, B> Future for ErrIntoFuture<T, U>
where
    T: Future<Output = One<Result<A, B>>>,
    B: Into<U>,
{
    type Output = One<Result<A, U>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future
            .poll()
            .map(|item| map_one(item, |x| x.map_err(Into::into)))
    }
}
