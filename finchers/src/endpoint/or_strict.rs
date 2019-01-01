use either::Either;

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, Poll};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrStrict<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> Endpoint for OrStrict<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Output = E1::Output>,
{
    type Output = E1::Output;
    type Future = OrStrictFuture<E1::Future, E2::Future>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let orig_cursor = ecx.cursor().clone();
        match self.e1.apply(ecx) {
            Ok(future1) => {
                *ecx.cursor() = orig_cursor;
                Ok(OrStrictFuture::left(future1))
            }
            Err(err1) => match self.e2.apply(ecx) {
                Ok(future) => Ok(OrStrictFuture::right(future)),
                Err(err2) => Err(err1.merge(err2)),
            },
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrStrictFuture<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrStrictFuture<L, R> {
    fn left(l: L) -> Self {
        OrStrictFuture {
            inner: Either::Left(l),
        }
    }

    fn right(r: R) -> Self {
        OrStrictFuture {
            inner: Either::Right(r),
        }
    }
}

impl<L, R> EndpointFuture for OrStrictFuture<L, R>
where
    L: EndpointFuture,
    R: EndpointFuture<Output = L::Output>,
{
    type Output = L::Output;

    #[inline]
    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        match self.inner {
            Either::Left(ref mut t) => t.poll_endpoint(cx),
            Either::Right(ref mut t) => t.poll_endpoint(cx),
        }
    }
}
