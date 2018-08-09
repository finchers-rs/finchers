#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{map_one, One};

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, A, B, U> EndpointBase for MapErr<E, F>
where
    E: EndpointBase<Output = One<Result<A, B>>>,
    F: FnOnce(B) -> U + Clone,
{
    type Output = One<Result<A, U>>;
    type Future = MapErrFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(MapErrFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapErrFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Future for MapErrFuture<T, F>
where
    T: Future<Output = One<Result<A, B>>>,
    F: FnOnce(B) -> U,
{
    type Output = One<Result<A, U>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            map_one(item, |x| x.map_err(f))
        })
    }
}
