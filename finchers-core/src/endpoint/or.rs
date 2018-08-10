#![allow(missing_docs)]

use futures_core::future::TryFuture;
use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use crate::either::Either;
use crate::endpoint::{Context, EndpointBase};
use crate::generic::{one, One};

#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> EndpointBase for Or<E1, E2>
where
    E1: EndpointBase,
    E2: EndpointBase,
{
    type Ok = One<Either<E1::Ok, E2::Ok>>;
    type Error = Either<E1::Error, E2::Error>;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&self, cx2: &mut Context) -> Option<Self::Future> {
        let mut cx1 = cx2.clone();
        let t1 = self.e1.apply(&mut cx1);
        let t2 = self.e2.apply(cx2);
        match (t1, t2) {
            (Some(t1), Some(t2)) => {
                // If both endpoints are matched, the one with the larger number of
                // (consumed) path segments is choosen.
                let res = if cx1.segments().popped() > cx2.segments().popped() {
                    *cx2 = cx1;
                    Either::Left(t1)
                } else {
                    Either::Right(t2)
                };
                Some(OrFuture { inner: res })
            }
            (Some(t1), None) => {
                *cx2 = cx1;
                Some(OrFuture {
                    inner: Either::Left(t1),
                })
            }
            (None, Some(t2)) => Some(OrFuture {
                inner: Either::Right(t2),
            }),
            (None, None) => None,
        }
    }
}

#[derive(Debug)]
pub struct OrFuture<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrFuture<L, R> {
    fn pinned_inner(self: PinMut<'a, Self>) -> Either<PinMut<'a, L>, PinMut<'a, R>> {
        match unsafe { &mut PinMut::get_mut_unchecked(self).inner } {
            Either::Left(ref mut t) => Either::Left(unsafe { PinMut::new_unchecked(t) }),
            Either::Right(ref mut t) => Either::Right(unsafe { PinMut::new_unchecked(t) }),
        }
    }
}

impl<L, R> Future for OrFuture<L, R>
where
    L: TryFuture,
    R: TryFuture,
{
    type Output = Result<One<Either<L::Ok, R::Ok>>, Either<L::Error, R::Error>>;

    #[inline(always)]
    fn poll(self: PinMut<Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        match self.pinned_inner() {
            Either::Left(t) => t
                .try_poll(cx)
                .map_ok(Either::Left)
                .map_ok(one)
                .map_err(Either::Left),
            Either::Right(t) => t
                .try_poll(cx)
                .map_ok(Either::Right)
                .map_ok(one)
                .map_err(Either::Right),
        }
    }
}
