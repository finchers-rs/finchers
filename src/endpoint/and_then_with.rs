use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use futures_util::try_future;
use futures_util::try_future::TryFutureExt;
use pin_utils::unsafe_pinned;

use super::try_chain::{TryChain, TryChainAction};
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AndThenWith<E, T, F> {
    pub(super) endpoint: E,
    pub(super) ctx: T,
    pub(super) f: F,
}

impl<'a, E, T, F, R> Endpoint<'a> for AndThenWith<E, T, F>
where
    E: Endpoint<'a>,
    T: 'a,
    F: Fn(&'a T, E::Output) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    type Output = (R::Ok,);
    type Future = AndThenWithFuture<'a, E::Future, T, F, R>;

    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        Ok(AndThenWithFuture {
            try_chain: TryChain::new(future, (&self.ctx, &self.f)),
        })
    }
}

#[derive(Debug)]
pub struct AndThenWithFuture<'a, Fut, T, F, R>
where
    Fut: TryFuture<Error = Error> + 'a,
    T: 'a,
    F: Fn(&'a T, Fut::Ok) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    try_chain: TryChain<Fut, try_future::MapOk<R, fn(R::Ok) -> (R::Ok,)>, (&'a T, &'a F)>,
}

impl<'a, Fut, T, F, R> AndThenWithFuture<'a, Fut, T, F, R>
where
    Fut: TryFuture<Error = Error> + 'a,
    T: 'a,
    F: Fn(&'a T, Fut::Ok) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    unsafe_pinned!(
        try_chain: TryChain<Fut, try_future::MapOk<R, fn(R::Ok) -> (R::Ok,)>, (&'a T, &'a F)>
    );
}

impl<'a, Fut, T, F, R> Future for AndThenWithFuture<'a, Fut, T, F, R>
where
    Fut: TryFuture<Error = Error> + 'a,
    T: 'a,
    F: Fn(&'a T, Fut::Ok) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    type Output = Result<(R::Ok,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.try_chain().poll(cx, |result, (ctx, f)| match result {
            Ok(out) => TryChainAction::Future(f(ctx, out).map_ok(|ok| (ok,))),
            Err(err) => TryChainAction::Output(Err(err)),
        })
    }
}
