use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint, IsEndpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, Poll, TryChain, TryChainAction};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for AndThen<E, F> {}

impl<E, F, Bd, R> Endpoint<Bd> for AndThen<E, F>
where
    E: Endpoint<Bd>,
    F: Func<E::Output, Out = R> + Clone,
    R: EndpointFuture<Bd>,
{
    type Output = (R::Output,);
    type Future = AndThenFuture<Bd, E::Future, F::Out, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(AndThenFuture {
            try_chain: TryChain::new(f1, self.f.clone()),
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct AndThenFuture<Bd, F1, F2, F>
where
    F1: EndpointFuture<Bd>,
    F2: EndpointFuture<Bd>,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    try_chain: TryChain<Bd, F1, F2, F>,
}

impl<Bd, F1, F2, F> EndpointFuture<Bd> for AndThenFuture<Bd, F1, F2, F>
where
    F1: EndpointFuture<Bd>,
    F2: EndpointFuture<Bd>,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    type Output = (F2::Output,);

    fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
        self.try_chain
            .try_poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok)),
                Err(err) => TryChainAction::Output(Err(err)),
            })
            .map(|x| x.map(|ok| (ok,)))
    }
}
