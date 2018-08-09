#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
use crate::generic::Tuple;

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, U> EndpointBase for MapErr<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Error) -> U + Clone,
{
    type Ok = E::Ok;
    type Error = U;
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

impl<T, F, U> Future for MapErrFuture<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> U,
{
    type Output = Result<T::Ok, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future
            .try_poll()
            .map_err(|err| (self.f.take().expect("cannot resolve twice"))(err))
    }
}
