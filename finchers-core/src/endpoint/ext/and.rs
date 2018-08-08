#![allow(missing_docs)]

use super::maybe_done::MaybeDone;
use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::future::{Future, Poll};
use std::fmt;

pub fn new<E1, E2>(e1: E1, e2: E2) -> And<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
{
    And {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> EndpointBase for And<E1, E2>
where
    E1: EndpointBase,
    E2: EndpointBase,
{
    type Output = (E1::Output, E2::Output);
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
{
    type Output = (F1::Output, F2::Output);

    fn poll(&mut self) -> Poll<Self::Output> {
        let mut all_done = self.f1.poll_done();
        all_done = all_done && self.f2.poll_done();

        if all_done {
            Poll::Ready((self.f1.take_item(), self.f2.take_item()))
        } else {
            Poll::Pending
        }
    }
}
