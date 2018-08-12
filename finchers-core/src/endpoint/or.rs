use futures_core::future::TryFuture;
use pin_utils::unsafe_pinned;
use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use either::Either;
use endpoint::EndpointBase;
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> EndpointBase for Or<E1, E2>
where
    E1: EndpointBase,
    E2: EndpointBase<Ok = E1::Ok>,
{
    type Ok = E1::Ok;
    type Error = Either<E1::Error, E2::Error>;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&self, mut input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
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

impl<L, R> Future for OrFuture<L, R>
where
    L: TryFuture,
    R: TryFuture<Ok = L::Ok>,
{
    type Output = Result<L::Ok, Either<L::Error, R::Error>>;

    #[inline(always)]
    fn poll(mut self: PinMut<Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        match self.inner().as_inner_pinned() {
            Either::Left(t) => t.try_poll(cx).map_err(Either::Left),
            Either::Right(t) => t.try_poll(cx).map_err(Either::Right),
        }
    }
}
