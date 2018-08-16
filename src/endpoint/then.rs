use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use futures_util::future::{FutureExt, Map};
use pin_utils::unsafe_pinned;

use crate::endpoint::Endpoint;
use crate::error::Error;
use crate::generic::{one, Func, One, Tuple};
use crate::input::{Cursor, Input};

use super::try_chain::{TryChain, TryChainAction};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Then<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> Endpoint for Then<E, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
    F::Out: Future,
{
    type Output = One<<F::Out as Future>::Output>;
    type Future = ThenFuture<E::Future, F::Out, F>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        let (f1, cursor) = self.endpoint.apply(input, cursor)?;
        let f = self.f.clone();
        Some((
            ThenFuture {
                try_chain: TryChain::new(f1, f),
            },
            cursor,
        ))
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ThenFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
{
    try_chain: TryChain<F1, Map<F2, fn(F2::Output) -> Result<F2::Output, Error>>, F>,
}

impl<F1, F2, F> ThenFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
{
    unsafe_pinned!(
        try_chain: TryChain<F1, Map<F2, fn(F2::Output) -> Result<F2::Output, Error>>, F>
    );
}

impl<F1, F2, F> Future for ThenFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
{
    type Output = Result<One<F2::Output>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.try_chain()
            .poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok).map(|x| Ok(x))),
                Err(err) => TryChainAction::Output(Err(err)),
            }).map_ok(one)
    }
}
