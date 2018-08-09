#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{map_one, One};

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, T, U> EndpointBase for OkOrElse<E, F>
where
    E: EndpointBase<Output = One<Option<T>>>,
    F: FnOnce() -> U + Clone,
{
    type Output = One<Result<T, U>>;
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
    T: Future<Output = One<Option<A>>>,
    F: FnOnce() -> U,
{
    type Output = One<Result<A, U>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            map_one(item, |x| x.ok_or_else(f))
        })
    }
}
