#![allow(missing_docs)]

use super::maybe_done::MaybeDone;
use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll};
use crate::generic::{Combine, Tuple};
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> EndpointBase for And<E1, E2>
where
    E1: EndpointBase,
    E2: EndpointBase,
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Future = AndFuture<E1::Future, E2::Future>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let f1 = self.e1.apply(cx)?;
        let f2 = self.e2.apply(cx)?;
        Some(AndFuture {
            f1: MaybeDone::Pending(f1),
            f2: MaybeDone::Pending(f2),
        })
    }
}

pub struct AndFuture<F1: Future, F2: Future> {
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<T1, T2> fmt::Debug for AndFuture<T1, T2>
where
    T1: Future + fmt::Debug,
    T2: Future + fmt::Debug,
    T1::Output: fmt::Debug,
    T2::Output: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AndFuture")
            .field("t1", &self.f1)
            .field("t2", &self.f2)
            .finish()
    }
}

impl<F1, F2> Future for AndFuture<F1, F2>
where
    F1: Future,
    F2: Future,
    F1::Output: Tuple,
    F2::Output: Tuple,
    F1::Output: Combine<F2::Output>,
{
    type Output = <F1::Output as Combine<F2::Output>>::Out;

    fn poll(&mut self) -> Poll<Self::Output> {
        let mut all_done = self.f1.poll_done();
        all_done = all_done && self.f2.poll_done();

        if all_done {
            Poll::Ready(Combine::combine(self.f1.take_item(), self.f2.take_item()))
        } else {
            Poll::Pending
        }
    }
}
