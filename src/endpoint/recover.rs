use std::future::Future;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use http::Response;
use pin_utils::unsafe_pinned;

use crate::common::Either;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::output::{Output, OutputContext};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Recover<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F, R> Endpoint<'a> for Recover<E, F>
where
    E: Endpoint<'a>,
    F: Fn(Error) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    type Output = (Recovered<E::Output, R::Ok>,);
    type Future = RecoverFuture<E::Future, R, &'a F>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(RecoverFuture {
            try_chain: TryChain::new(f1, &self.f),
        })
    }
}

#[derive(Debug)]
pub struct Recovered<L, R>(Either<L, R>);

impl<L: Output, R: Output> Output for Recovered<L, R> {
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    #[inline(always)]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(cx)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct RecoverFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: FnOnce(Error) -> F2,
{
    try_chain: TryChain<F1, F2, F>,
}

impl<F1, F2, F> RecoverFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: FnOnce(Error) -> F2,
{
    unsafe_pinned!(try_chain: TryChain<F1, F2, F>);
}

impl<F1, F2, F> Future for RecoverFuture<F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
    F: FnOnce(Error) -> F2,
{
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    type Output = Result<(Recovered<F1::Ok, F2::Ok>,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.try_chain()
            .poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Output(Ok(Either::Left(ok))),
                Err(err) => TryChainAction::Future(f(err)),
            }).map_ok(|ok| (Recovered(ok),))
    }
}

#[derive(Debug)]
enum TryChain<F1, F2, T> {
    First(F1, Option<T>),
    Second(F2),
    Empty,
}

pub(super) enum TryChainAction<F1, F2>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
{
    Future(F2),
    Output(Result<Either<F1::Ok, F2::Ok>, Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: TryFuture<Error = Error>,
    F2: TryFuture<Error = Error>,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    pub(super) fn poll<F>(
        self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
        f: F,
    ) -> Poll<Result<Either<F1::Ok, F2::Ok>, Error>>
    where
        F: FnOnce(Result<F1::Ok, F1::Error>, T) -> TryChainAction<F1, F2>,
    {
        let mut f = Some(f);

        // Safety: the futures does not move in this method.
        let this = unsafe { PinMut::get_mut_unchecked(self) };

        loop {
            let (out, data) = match this {
                TryChain::First(f1, data) => {
                    match unsafe { PinMut::new_unchecked(f1) }.try_poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(out) => (out, data.take().unwrap()),
                    }
                }
                TryChain::Second(f2) => {
                    return unsafe { PinMut::new_unchecked(f2) }
                        .try_poll(cx)
                        .map_ok(Either::Right)
                }
                TryChain::Empty => panic!("This future has already polled."),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Future(f2) => {
                    *this = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *this = TryChain::Empty;
                    return Poll::Ready(out);
                }
            }
        }
    }
}
