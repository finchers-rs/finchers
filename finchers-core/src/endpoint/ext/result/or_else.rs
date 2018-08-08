#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};

#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> OrElse<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U> + Clone,
{
    OrElse { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for OrElse<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U> + Clone,
{
    type Output = Result<A, U>;
    type Future = OrElseFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(OrElseFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OrElseFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Future for OrElseFuture<T, F>
where
    T: Future<Output = Result<A, B>>,
    F: FnOnce(B) -> Result<A, U>,
{
    type Output = Result<A, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.or_else(f)
        })
    }
}
