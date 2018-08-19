use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use http::Response;
use pin_utils::unsafe_pinned;

use crate::endpoint::{Context, Endpoint, EndpointErrorKind, EndpointResult};
use crate::error::Error;
use crate::generic::{one, Either, One};
use crate::input::Input;
use crate::output::Responder;

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Output = One<WrappedEither<E1::Output, E2::Output>>;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match {
            let mut ecx = ecx.clone_reborrowed();
            let res = self.e1.apply(&mut ecx);
            res.map(|future| (future, ecx.current_cursor()))
        } {
            Ok((future1, cursor1)) => {
                match {
                    let mut ecx = ecx.clone_reborrowed();
                    let res = self.e2.apply(&mut ecx);
                    res.map(|future| (future, ecx.current_cursor()))
                } {
                    // If both endpoints are matched, the one with the larger number of
                    // (consumed) path segments is choosen.
                    Ok((_, ref cursor2)) if cursor1.popped >= cursor2.popped => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture {
                            inner: Either::Left(future1),
                        })
                    }
                    Ok((future2, cursor2)) => {
                        ecx.reset_cursor(cursor2);
                        Ok(OrFuture {
                            inner: Either::Right(future2),
                        })
                    }
                    Err(..) => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture {
                            inner: Either::Left(future1),
                        })
                    }
                }
            }
            Err(err1) => match self.e2.apply(ecx) {
                Err(EndpointErrorKind::MethodNotAllowed(allows2)) => match err1 {
                    EndpointErrorKind::MethodNotAllowed(mut allows1) => {
                        allows1.extend(allows2);
                        Err(EndpointErrorKind::MethodNotAllowed(allows1))
                    }
                    EndpointErrorKind::NotMatched => {
                        Err(EndpointErrorKind::MethodNotAllowed(allows2))
                    }
                },
                Err(EndpointErrorKind::NotMatched) => Err(err1),
                Ok(future) => Ok(OrFuture {
                    inner: Either::Right(future),
                }),
            },
        }
    }
}

#[derive(Debug)]
pub struct WrappedEither<L, R>(Either<L, R>);

impl<L, R> Responder for WrappedEither<L, R>
where
    L: Responder,
    R: Responder,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    #[inline(always)]
    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(input)
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
    L: TryFuture<Error = Error>,
    R: TryFuture<Error = Error>,
{
    type Output = Result<One<WrappedEither<L::Ok, R::Ok>>, Error>;

    #[inline(always)]
    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner().as_pin_mut() {
            Either::Left(t) => t
                .try_poll(cx)
                .map_ok(|t| one(WrappedEither(Either::Left(t)))),
            Either::Right(t) => t
                .try_poll(cx)
                .map_ok(|t| one(WrappedEither(Either::Right(t)))),
        }
    }
}
