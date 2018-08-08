#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::future::{Future, Poll};
use std::mem;

pub fn new<E, F, R>(endpoint: E, f: F) -> Then<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Output) -> R + Clone,
    R: Future,
{
    Then {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Then<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> EndpointBase for Then<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Output) -> R + Clone,
    R: Future,
{
    type Output = R::Output;
    type Future = ThenFuture<E::Future, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        let f = self.f.clone();
        Some(ThenFuture::First(future, f))
    }
}

#[derive(Debug)]
pub enum ThenFuture<T, F, R>
where
    T: Future,
    F: FnOnce(T::Output) -> R,
    R: Future,
{
    First(T, F),
    Second(R),
    Done,
}

impl<T, F, R> Future for ThenFuture<T, F, R>
where
    T: Future,
    F: FnOnce(T::Output) -> R,
    R: Future,
{
    type Output = R::Output;

    fn poll(&mut self) -> Poll<Self::Output> {
        use self::ThenFuture::*;
        loop {
            // TODO: optimize
            match mem::replace(self, Done) {
                First(mut task, f) => match task.poll() {
                    Poll::Pending => {
                        *self = First(task, f);
                        return Poll::Pending;
                    }
                    Poll::Ready(r) => {
                        *self = Second(f(r));
                        continue;
                    }
                },
                Second(mut fut) => {
                    return match fut.poll() {
                        Poll::Pending => {
                            *self = Second(fut);
                            Poll::Pending
                        }
                        polled => polled,
                    }
                }
                Done => panic!(),
            }
        }
    }
}
