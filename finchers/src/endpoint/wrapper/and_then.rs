use std::marker::PhantomData;

use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{EndpointFuture, Poll, TaskContext, TryChain, TryChainAction};

use super::Wrapper;

/// Create a wrapper for creating an endpoint which executes another future
/// created by the specified function after the precedent future resolves.
pub fn and_then<T, F>(f: F) -> AndThen<T, F>
where
    T: Tuple,
    F: Func<T>,
    F::Out: EndpointFuture,
{
    AndThen {
        f,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AndThen<T, F> {
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<E, F, R> Wrapper<E> for AndThen<E::Output, F>
where
    E: Endpoint,
    F: Func<E::Output, Out = R> + Clone,
    R: EndpointFuture,
{
    type Output = (R::Output,);
    type Endpoint = AndThenEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        AndThenEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AndThenEndpoint<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for AndThenEndpoint<E, F>
where
    E: Endpoint,
    F: Func<E::Output, Out = R> + Clone,
    R: EndpointFuture,
{
    type Output = (R::Output,);
    type Future = AndThenFuture<E::Future, F::Out, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(AndThenFuture {
            try_chain: TryChain::new(f1, self.f.clone()),
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct AndThenFuture<F1, F2, F>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    try_chain: TryChain<F1, F2, F>,
}

impl<F1, F2, F> EndpointFuture for AndThenFuture<F1, F2, F>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    type Output = (F2::Output,);

    fn poll_endpoint(&mut self, cx: &mut TaskContext<'_>) -> Poll<Self::Output, Error> {
        self.try_chain
            .try_poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok)),
                Err(err) => TryChainAction::Output(Err(err)),
            })
            .map(|x| x.map(|ok| (ok,)))
    }
}
