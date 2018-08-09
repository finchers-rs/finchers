#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{Func, Tuple};

#[derive(Copy, Clone, Debug)]
pub struct Map<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for Map<E, F>
where
    E: EndpointBase,
    F: Func<E::Output> + Clone,
    F::Out: Tuple,
{
    type Output = F::Out;
    type Future = MapFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(MapFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> Future for MapFuture<T, F>
where
    T: Future,
    T::Output: Tuple,
    F: Func<T::Output>,
    F::Out: Tuple,
{
    type Output = F::Out;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            f.call(item)
        })
    }
}
