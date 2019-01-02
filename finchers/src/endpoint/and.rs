use crate::common::{Combine, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, MaybeDone, Poll};

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
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Future = AndFuture<E1::Future, E2::Future>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(AndFuture {
            f1: self.e1.apply(ecx).map(MaybeDone::Pending)?,
            f2: self.e2.apply(ecx).map(MaybeDone::Pending)?,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct AndFuture<F1, F2>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
{
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<F1, F2> EndpointFuture for AndFuture<F1, F2>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F1::Output: Combine<F2::Output>,
    F2::Output: Tuple,
{
    type Output = <F1::Output as Combine<F2::Output>>::Out;

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        futures::try_ready!(self.f1.poll_endpoint(cx));
        futures::try_ready!(self.f2.poll_endpoint(cx));
        let v1 = self
            .f1
            .take_item()
            .expect("the future has already been polled.");
        let v2 = self
            .f2
            .take_item()
            .expect("the future has already been polled.");
        Ok(Combine::combine(v1, v2).into())
    }
}
