use either::Either;

use endpoint::{Context, Endpoint, EndpointResult};
use error::Error;

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrStrict<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<'a, E1, E2> Endpoint<'a> for OrStrict<E1, E2>
where
    E1: Endpoint<'a>,
    E2: Endpoint<'a, Output = E1::Output>,
{
    type Output = E1::Output;
    type Future = OrStrictFuture<E1::Future, E2::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match {
            let mut ecx = ecx.clone_reborrowed();
            self.e1
                .apply(&mut ecx)
                .map(|future| (future, ecx.current_cursor()))
        } {
            Ok((future1, cursor1)) => {
                ecx.reset_cursor(cursor1);
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

impl<L, R> ::futures::Future for OrStrictFuture<L, R>
where
    L: ::futures::Future<Error = Error>,
    R: ::futures::Future<Item = L::Item, Error = Error>,
{
    type Item = L::Item;
    type Error = Error;

    #[inline(always)]
    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        match self.inner {
            Either::Left(ref mut t) => t.poll(),
            Either::Right(ref mut t) => t.poll(),
        }
    }
}
