#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
use crate::generic::{Func, Tuple};

#[derive(Debug, Copy, Clone)]
pub struct MapOk<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for MapOk<E, F>
where
    E: EndpointBase,
    F: Func<E::Ok> + Clone,
    F::Out: Tuple,
{
    type Ok = F::Out;
    type Error = E::Error;
    type Future = MapOkFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(MapOkFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapOkFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> Future for MapOkFuture<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: Tuple,
{
    type Output = Result<F::Out, T::Error>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future
            .try_poll()
            .map_ok(|item| self.f.take().expect("cannot resolve twice").call(item))
    }
}
