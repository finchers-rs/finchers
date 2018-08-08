use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::future::{Future, Poll};

pub fn new<E>(endpoint: E) -> Lift<E::Endpoint>
where
    E: IntoEndpoint,
{
    Lift {
        endpoint: endpoint.into_endpoint(),
    }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Lift<E> {
    endpoint: E,
}

impl<E> EndpointBase for Lift<E>
where
    E: EndpointBase,
{
    type Output = Option<E::Output>;
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
    type Output = Option<T::Output>;

    fn poll(&mut self) -> Poll<Self::Output> {
        match self.future {
            Some(ref mut t) => t.poll().map(Some),
            None => Poll::Ready(None),
        }
    }
}
