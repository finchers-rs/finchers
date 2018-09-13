use std::marker::PhantomData;
use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use futures_util::future::{FutureExt, Map};
use pin_utils::unsafe_pinned;

use crate::common::{Func, Tuple};
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

use super::try_chain::{TryChain, TryChainAction};
use super::Wrapper;

/// Create a wrapper for creating an endpoint which executes another future
/// created by the specified function after the precedent future resolves.
///
/// Unlike `and_then`, the future created by the specified function does not
/// return the error value.
pub fn then<T, F>(f: F) -> Then<T, F>
where
    T: Tuple,
    F: Func<T>,
    F::Out: Future,
{
    Then {
        f,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Then<T, F> {
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<'a, E, F> Wrapper<'a, E> for Then<E::Output, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
    F::Out: Future + 'a,
{
    type Output = (<F::Out as Future>::Output,);
    type Endpoint = ThenEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        ThenEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ThenEndpoint<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F> Endpoint<'a> for ThenEndpoint<E, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
    F::Out: Future + 'a,
{
    type Output = (<F::Out as Future>::Output,);
    type Future = ThenFuture<'a, E::Future, F::Out, F>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(ThenFuture {
            try_chain: TryChain::new(f1, &self.f),
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    try_chain: TryChain<F1, Map<F2, fn(F2::Output) -> Result<F2::Output, Error>>, &'a F>,
}

impl<'a, F1, F2, F> ThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    unsafe_pinned!(
        try_chain: TryChain<F1, Map<F2, fn(F2::Output) -> Result<F2::Output, Error>>, &'a F>
    );
}

impl<'a, F1, F2, F> Future for ThenFuture<'a, F1, F2, F>
where
    F1: TryFuture<Error = Error>,
    F2: Future,
    F: Func<F1::Ok, Out = F2> + 'a,
    F1::Ok: Tuple,
{
    type Output = Result<(F2::Output,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.try_chain()
            .poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok).map(Ok)),
                Err(err) => TryChainAction::Output(Err(err)),
            }).map_ok(|x| (x,))
    }
}
