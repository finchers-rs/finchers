use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{one, One};

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Lift<E> {
    pub(super) endpoint: E,
}

impl<E> EndpointBase for Lift<E>
where
    E: EndpointBase,
{
    type Output = One<Option<E::Output>>;
    type Future = LiftFuture<E::Future>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(LiftFuture {
            future: self.endpoint.apply(cx),
        })
    }
}

#[derive(Debug)]
pub struct LiftFuture<T> {
    future: Option<T>,
}

impl<T> Future for LiftFuture<T>
where
    T: Future,
{
    type Output = One<Option<T::Output>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        match self.future {
            Some(ref mut t) => t.poll().map(Some).map(one),
            None => Poll::Ready(one(None)),
        }
    }
}
