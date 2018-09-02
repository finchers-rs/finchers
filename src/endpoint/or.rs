use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use pin_utils::unsafe_pinned;

use http::Response;

use crate::common::Either;
use crate::common::Either::*;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::output::{Output, OutputContext};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<'a, E1, E2> Endpoint<'a> for Or<E1, E2>
where
    E1: Endpoint<'a>,
    E2: Endpoint<'a>,
{
    type Output = (Wrapped<E1::Output, E2::Output>,);
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match {
            let mut ecx = ecx.clone_reborrowed();
            self.e1
                .apply(&mut ecx)
                .map(|future| (future, ecx.current_cursor()))
        } {
            Ok((future1, cursor1)) => {
                match {
                    let mut ecx = ecx.clone_reborrowed();
                    self.e2
                        .apply(&mut ecx)
                        .map(|future| (future, ecx.current_cursor()))
                } {
                    // If both endpoints are matched, the one with the larger number of
                    // (consumed) path segments is choosen.
                    Ok((_, ref cursor2)) if cursor1.popped >= cursor2.popped => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture::left(future1))
                    }
                    Ok((future2, cursor2)) => {
                        ecx.reset_cursor(cursor2);
                        Ok(OrFuture::right(future2))
                    }
                    Err(..) => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture::left(future1))
                    }
                }
            }
            Err(err1) => match self.e2.apply(ecx) {
                Ok(future) => Ok(OrFuture::right(future)),
                Err(err2) => Err(err1.merge(&err2)),
            },
        }
    }
}

#[derive(Debug)]
pub struct Wrapped<L, R>(Either<L, R>);

impl<L: Output, R: Output> Output for Wrapped<L, R> {
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    #[inline(always)]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(cx)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrFuture<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrFuture<L, R> {
    fn left(l: L) -> Self {
        OrFuture {
            inner: Either::Left(l),
        }
    }

    fn right(r: R) -> Self {
        OrFuture {
            inner: Either::Right(r),
        }
    }

    unsafe_pinned!(inner: Either<L, R>);
}

impl<L, R> Future for OrFuture<L, R>
where
    L: TryFuture<Error = Error>,
    R: TryFuture<Error = Error>,
{
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    type Output = Result<(Wrapped<L::Ok, R::Ok>,), Error>;

    #[inline(always)]
    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner().as_pin_mut() {
            Left(t) => t.try_poll(cx).map_ok(|t| (Wrapped(Left(t)),)),
            Right(t) => t.try_poll(cx).map_ok(|t| (Wrapped(Right(t)),)),
        }
    }
}
