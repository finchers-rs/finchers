#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapErr<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> U + Clone,
{
    MapErr { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for MapErr<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(B) -> U + Clone,
{
    type Output = Result<A, U>;
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
    T: Future<Output = Result<A, B>>,
    F: FnOnce(B) -> U,
{
    type Output = Result<A, U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map_err(f)
        })
    }
}
