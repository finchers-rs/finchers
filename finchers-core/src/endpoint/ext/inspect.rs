#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::future::{Future, Poll};

pub fn new<E, F>(endpoint: E, f: F) -> Inspect<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(&E::Output) + Clone,
{
    Inspect {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Inspect<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F> EndpointBase for Inspect<E, F>
where
    E: EndpointBase,
    F: FnOnce(&E::Output) + Clone,
{
    type Output = E::Output;
    type Future = InspectFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(InspectFuture {
            future: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct InspectFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> Future for InspectFuture<T, F>
where
    T: Future,
    F: FnOnce(&T::Output),
{
    type Output = T::Output;

    fn poll(&mut self) -> Poll<Self::Output> {
        self.future.poll().map(|item| {
            let f = self.f.take().expect("cannot resolve twice");
            f(&item);
            item
        })
    }
}
