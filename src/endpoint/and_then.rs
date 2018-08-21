use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use pin_utils::unsafe_pinned;

use crate::common::{Func, Tuple};
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

use super::try_chain::{TryChain, TryChainAction};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F> Endpoint<'a> for AndThen<E, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
    F::Out: TryFuture<Error = Error>,
{
    type Output = (<F::Out as TryFuture>::Ok,);
    type Future = AndThenFuture<'a, E::Future, F::Out, F>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(AndThenFuture {
            try_chain: TryChain::new(f1, &self.f),
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AndThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    try_chain: TryChain<F1, F2, &'a F>,
}

impl<'a, F1, F2, F> AndThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    unsafe_pinned!(try_chain: TryChain<F1, F2, &'a F>);
}

impl<'a, F1, F2, F> Future for AndThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    type Output = Result<(F2::Ok,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.try_chain()
            .poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok)),
                Err(err) => TryChainAction::Output(Err(err)),
            }).map_ok(|ok| (ok,))
    }
}
