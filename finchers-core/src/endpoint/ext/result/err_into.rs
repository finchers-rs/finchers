#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

pub fn new<E, U, A, B>(endpoint: E) -> ErrInto<E, U>
where
    E: EndpointBase<Output = Result<A, B>>,
    B: Into<U>,
{
    ErrInto {
        endpoint,
        _marker: PhantomData,
    }
}

impl<E, A, B, U> EndpointBase for ErrInto<E, U>
where
    E: EndpointBase<Output = Result<A, B>>,
    B: Into<U>,
{
    type Output = Result<A, U>;
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
    T: Future<Output = Result<A, B>>,
    B: Into<U>,
{
    type Output = Result<A, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| item.map_err(Into::into))
    }
}
