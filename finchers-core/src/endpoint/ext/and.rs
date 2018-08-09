#![allow(missing_docs)]

use super::maybe_done::MaybeDone;
use crate::either::Either;
use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
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
    E1::Ok: Combine<E2::Ok>,
{
    type Ok = <E1::Ok as Combine<E2::Ok>>::Out;
    type Error = Either<E1::Error, E2::Error>;
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

pub struct AndFuture<F1: TryFuture, F2: TryFuture> {
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<T1, T2> fmt::Debug for AndFuture<T1, T2>
where
    T1: TryFuture + fmt::Debug,
    T2: TryFuture + fmt::Debug,
    T1::Ok: fmt::Debug,
    T2::Ok: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AndFuture")
            .field("f1", &self.f1)
            .field("f2", &self.f2)
            .finish()
    }
}

impl<F1, F2> Future for AndFuture<F1, F2>
where
    F1: TryFuture,
    F2: TryFuture,
    F1::Ok: Tuple + Combine<F2::Ok>,
    F2::Ok: Tuple,
{
    type Output = Result<<F1::Ok as Combine<F2::Ok>>::Out, Either<F1::Error, F2::Error>>;

    fn poll(&mut self) -> Poll<Self::Output> {
        let mut all_done = match self.f1.poll_done() {
            Ok(all_done) => all_done,
            Err(err) => return Poll::Ready(Err(Either::Left(err))),
        };
        all_done = match self.f2.poll_done() {
            Ok(done) => all_done && done,
            Err(err) => return Poll::Ready(Err(Either::Right(err))),
        };

        if !all_done {
            return Poll::Pending;
        }

        Poll::Ready(Ok(Combine::combine(
            self.f1.take_item(),
            self.f2.take_item(),
        )))
    }
}
