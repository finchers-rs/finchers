#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};

#[derive(Debug, Copy, Clone)]
pub struct MapSome<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, T>(endpoint: E, f: F) -> MapSome<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone,
{
    MapSome { endpoint, f }
}

impl<E, F, T, U> EndpointBase for MapSome<E, F>
where
    E: EndpointBase<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone,
{
    type Output = Option<U>;
    type Future = MapSomeFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(MapSomeFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapSomeFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, A, U> Future for MapSomeFuture<T, F>
where
    T: Future<Output = Option<A>>,
    F: FnOnce(A) -> U,
{
    type Output = Option<U>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            item.map(f)
        })
    }
}
