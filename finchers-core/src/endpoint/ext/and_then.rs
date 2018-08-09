#![allow(missing_docs)]

use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use pin_utils::unsafe_pinned;

use super::try_chain::{TryChain, TryChainAction};
use crate::endpoint::{Context, EndpointBase};
use crate::generic::{Func, Tuple};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for AndThen<E, F>
where
    E: EndpointBase,
    F: Func<E::Ok> + Clone,
    F::Out: TryFuture<Error = E::Error>,
    <F::Out as TryFuture>::Ok: Tuple,
{
    type Ok = <F::Out as TryFuture>::Ok;
    type Error = E::Error;
    type Future = AndThenFuture<E::Future, F::Out, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let f1 = self.endpoint.apply(cx)?;
        let f = self.f.clone();
        Some(AndThenFuture {
            try_chain: TryChain::new(f1, f),
        })
    }
}

#[derive(Debug)]
pub struct AndThenFuture<F1, F2, F>
where
    F1: TryFuture,
    F2: TryFuture<Error = F1::Error>,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
    <F::Out as TryFuture>::Ok: Tuple,
{
    try_chain: TryChain<F1, F2, F>,
}

impl<F1, F2, F> AndThenFuture<F1, F2, F>
where
    F1: TryFuture,
    F2: TryFuture<Error = F1::Error>,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
    <F::Out as TryFuture>::Ok: Tuple,
{
    unsafe_pinned!(try_chain: TryChain<F1, F2, F>);
}

impl<F1, F2, F> Future for AndThenFuture<F1, F2, F>
where
    F1: TryFuture,
    F2: TryFuture<Error = F1::Error>,
    F: Func<F1::Ok, Out = F2>,
    F1::Ok: Tuple,
    <F::Out as TryFuture>::Ok: Tuple,
{
    type Output = Result<F2::Ok, F2::Error>;

    fn poll(mut self: PinMut<Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        self.try_chain().poll(cx, |result, f| match result {
            Ok(ok) => TryChainAction::Future(f.call(ok)),
            Err(err) => TryChainAction::Output(Err(err)),
        })
    }
}
