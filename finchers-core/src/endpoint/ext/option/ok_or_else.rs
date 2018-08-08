#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, T, U>(endpoint: E, f: F) -> OkOrElse<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce() -> U + Clone,
{
    OkOrElse { endpoint, f }
}

impl<E, F, T, U> EndpointBase for OkOrElse<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce() -> U + Clone,
{
    type Output = Result<T, U>;
    type Future = OkOrElseFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(OkOrElseFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OkOrElseFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, A, U> Future for OkOrElseFuture<T, F>
where
    T: Future<Output = Option<A>>,
    F: FnOnce() -> U,
{
    type Output = Result<A, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item: Option<A>| {
            let f = self.f.take().expect("cannot resolve twice");
            item.ok_or_else(f)
        })
    }
}
