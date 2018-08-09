#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{Func, Tuple};
use std::mem;

#[derive(Copy, Clone, Debug)]
pub struct Then<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for Then<E, F>
where
    E: EndpointBase,
    F: Func<E::Output> + Clone,
    F::Out: Future,
    <F::Out as Future>::Output: Tuple,
{
    type Output = <F::Out as Future>::Output;
    type Future = ThenFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        let f = self.f.clone();
        Some(ThenFuture::First(future, f))
    }
}

#[derive(Debug)]
pub enum ThenFuture<T, F>
where
    T: Future,
    T::Output: Tuple,
    F: Func<T::Output>,
    F::Out: Future,
    <F::Out as Future>::Output: Tuple,
{
    First(T, F),
    Second(F::Out),
    Done,
}

impl<T, F> Future for ThenFuture<T, F>
where
    T: Future,
    T::Output: Tuple,
    F: Func<T::Output>,
    F::Out: Future,
    <F::Out as Future>::Output: Tuple,
{
    type Output = <F::Out as Future>::Output;

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
                        *self = Second(f.call(r));
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
