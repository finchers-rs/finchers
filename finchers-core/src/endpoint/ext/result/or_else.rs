#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{map_one, One};

#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, A, B, U> EndpointBase for OrElse<E, F>
where
    E: EndpointBase<Output = One<Result<A, B>>>,
    F: FnOnce(B) -> Result<A, U> + Clone,
{
    type Output = One<Result<A, U>>;
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
    T: Future<Output = One<Result<A, B>>>,
    F: FnOnce(B) -> Result<A, U>,
{
    type Output = One<Result<A, U>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            map_one(item, |x| x.or_else(f))
        })
    }
}
