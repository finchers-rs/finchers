use futures_core::future::TryFuture;
use pin_utils::unsafe_pinned;
use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use endpoint::Endpoint;
use error::Error;
use generic::Either;
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Output = E1::Output>,
{
    type Output = E1::Output;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(
        &self,
        mut input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        let v1 = self.e1.apply(input.reborrow(), cursor.clone());
        let v2 = self.e2.apply(input, cursor);

        match (v1, v2) {
            (Some((future1, cursor1)), Some((future2, cursor2))) => {
                // If both endpoints are matched, the one with the larger number of
                // (consumed) path segments is choosen.
                if cursor1.popped() >= cursor2.popped() {
                    Some((
                        OrFuture {
                            inner: Either::Left(future1),
                        },
                        cursor1,
                    ))
                } else {
                    Some((
                        OrFuture {
                            inner: Either::Right(future2),
                        },
                        cursor2,
                    ))
                }
            }
            (Some((future, cursor)), None) => Some((
                OrFuture {
                    inner: Either::Left(future),
                },
                cursor,
            )),
            (None, Some((future, cursor))) => Some((
                OrFuture {
                    inner: Either::Right(future),
                },
                cursor,
            )),
            (None, None) => None,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrFuture<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrFuture<L, R> {
    unsafe_pinned!(inner: Either<L, R>);
}

impl<L, R, T> Future for OrFuture<L, R>
where
    L: TryFuture<Ok = T, Error = Error>,
    R: TryFuture<Ok = T, Error = Error>,
{
    type Output = Result<T, Error>;

    #[inline(always)]
    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner().as_pin_mut() {
            Either::Left(t) => t.try_poll(cx),
            Either::Right(t) => t.try_poll(cx),
        }
    }
}
