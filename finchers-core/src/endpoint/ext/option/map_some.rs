#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{map_one, One};

#[derive(Debug, Copy, Clone)]
pub struct MapSome<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, T, U> EndpointBase for MapSome<E, F>
where
    E: EndpointBase<Output = One<Option<T>>>,
    F: FnOnce(T) -> U + Clone,
{
    type Output = One<Option<U>>;
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
    T: Future<Output = One<Option<A>>>,
    F: FnOnce(A) -> U,
{
    type Output = One<Option<U>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            map_one(item, |x| x.map(f))
        })
    }
}
