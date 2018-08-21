use futures_core::future::Future;
use futures_util::future::{maybe_done, MaybeDone};
use futures_util::try_future::{IntoFuture, TryFutureExt};
use pin_utils::unsafe_pinned;
use std::fmt;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use crate::common::{Combine, Tuple};
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<'a, E1, E2> Endpoint<'a> for And<E1, E2>
where
    E1: Endpoint<'a>,
    E2: Endpoint<'a>,
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Future = AndFuture<IntoFuture<E1::Future>, IntoFuture<E2::Future>>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let f1 = self.e1.apply(ecx)?;
        let f2 = self.e2.apply(ecx)?;
        Ok(AndFuture {
            f1: maybe_done(f1.into_future()),
            f2: maybe_done(f2.into_future()),
        })
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<F1, F2, T1, T2> Future for AndFuture<F1, F2>
where
    F1: Future<Output = Result<T1, Error>>,
    F2: Future<Output = Result<T2, Error>>,
    T1: Tuple + Combine<T2>,
    T2: Tuple,
{
    type Output = Result<<T1 as Combine<T2>>::Out, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        // FIXME: early return if MaybeDone::poll(cx) returns an Err.
        let all_done = !self.f1().poll(cx).is_pending() && !self.f2().poll(cx).is_pending();
        if all_done {
            let v1 = match self.f1().take_output().unwrap() {
                Ok(v) => v,
                Err(e) => return Poll::Ready(Err(e)),
            };
            let v2 = match self.f2().take_output().unwrap() {
                Ok(v) => v,
                Err(e) => return Poll::Ready(Err(e)),
            };
            Poll::Ready(Ok(Combine::combine(v1, v2)))
        } else {
            Poll::Pending
        }
    }
}
