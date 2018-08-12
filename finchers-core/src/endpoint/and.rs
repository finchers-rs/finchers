use futures_core::future::Future;
use futures_util::future::{maybe_done, MaybeDone};
use futures_util::try_future::{IntoFuture, TryFutureExt};
use pin_utils::unsafe_pinned;
use std::fmt;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use either::Either;
use endpoint::Endpoint;
use generic::{Combine, Tuple};
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> Endpoint for And<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
    E1::Ok: Combine<E2::Ok>,
{
    type Ok = <E1::Ok as Combine<E2::Ok>>::Out;
    type Error = Either<E1::Error, E2::Error>;
    type Future = AndFuture<IntoFuture<E1::Future>, IntoFuture<E2::Future>>;

    fn apply(&self, mut input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (f1, cursor) = self.e1.apply(input.reborrow(), cursor)?;
        let (f2, cursor) = self.e2.apply(input, cursor)?;
        Some((
            AndFuture {
                f1: maybe_done(f1.into_future()),
                f2: maybe_done(f2.into_future()),
            },
            cursor,
        ))
    }
}

pub struct AndFuture<F1: Future, F2: Future> {
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<F1, F2> fmt::Debug for AndFuture<F1, F2>
where
    F1: Future + fmt::Debug,
    F2: Future + fmt::Debug,
    F1::Output: fmt::Debug,
    F2::Output: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("AndFuture")
            .field("f1", &self.f1)
            .field("f2", &self.f2)
            .finish()
    }
}

impl<F1: Future, F2: Future> AndFuture<F1, F2> {
    unsafe_pinned!(f1: MaybeDone<F1>);
    unsafe_pinned!(f2: MaybeDone<F2>);
}

impl<F1, F2, T1, T2, E1, E2> Future for AndFuture<F1, F2>
where
    F1: Future<Output = Result<T1, E1>>,
    F2: Future<Output = Result<T2, E2>>,
    T1: Tuple + Combine<T2>,
    T2: Tuple,
{
    type Output = Result<<T1 as Combine<T2>>::Out, Either<E1, E2>>;

    fn poll(mut self: PinMut<Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        // FIXME: early return if MaybeDone::poll(cx) returns an Err.
        let mut all_done = true;
        if self.f1().poll(cx).is_pending() {
            all_done = false;
        }
        if self.f2().poll(cx).is_pending() {
            all_done = false;
        }

        if all_done {
            let v1 = match self.f1().take_output().unwrap() {
                Ok(v) => v,
                Err(e) => return Poll::Ready(Err(Either::Left(e))),
            };
            let v2 = match self.f2().take_output().unwrap() {
                Ok(v) => v,
                Err(e) => return Poll::Ready(Err(Either::Right(e))),
            };
            Poll::Ready(Ok(Combine::combine(v1, v2)))
        } else {
            Poll::Pending
        }
    }
}
