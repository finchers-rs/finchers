#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};

#[derive(Debug, Copy, Clone)]
pub struct MapOk<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> MapOk<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone,
{
    MapOk { endpoint, f }
}

impl<E, F, A, B, U> EndpointBase for MapOk<E, F>
where
    E: EndpointBase<Output = Result<A, B>>,
    F: FnOnce(A) -> U + Clone,
{
    type Output = Result<U, B>;
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

impl<T, F, U, A, B> Future for MapOkFuture<T, F>
where
    T: Future<Output = Result<A, B>>,
    F: FnOnce(A) -> U,
{
    type Output = Result<U, B>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map(f)
        })
    }
}
