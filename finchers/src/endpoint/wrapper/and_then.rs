use std::marker::PhantomData;

use futures::{Future, IntoFuture, Poll};

use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;

use super::try_chain::{TryChain, TryChainAction};
use super::Wrapper;

/// Create a wrapper for creating an endpoint which executes another future
/// created by the specified function after the precedent future resolves.
pub fn and_then<T, F>(f: F) -> AndThen<T, F>
where
    T: Tuple,
    F: Func<T>,
    F::Out: IntoFuture<Error = Error>,
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

impl<E, F> Wrapper<E> for AndThen<E::Output, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
    F::Out: IntoFuture<Error = Error>,
{
    type Output = (<F::Out as IntoFuture>::Item,);
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

impl<E, F> Endpoint for AndThenEndpoint<E, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
    F::Out: IntoFuture<Error = Error>,
{
    type Output = (<F::Out as IntoFuture>::Item,);
    type Future = AndThenFuture<E::Future, F::Out, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(AndThenFuture {
            try_chain: TryChain::new(f1, self.f.clone()),
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AndThenFuture<F1, F2, F>
where
    F1: Future<Error = Error>,
    F2: IntoFuture<Error = Error>,
    F: Func<F1::Item, Out = F2>,
    F1::Item: Tuple,
{
    try_chain: TryChain<F1, F2::Future, F>,
}

impl<F1, F2, F> Future for AndThenFuture<F1, F2, F>
where
    F1: Future<Error = Error>,
    F2: IntoFuture<Error = Error>,
    F: Func<F1::Item, Out = F2>,
    F1::Item: Tuple,
{
    type Item = (F2::Item,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.try_chain
            .poll(|result, f| match result {
                Ok(ok) => TryChainAction::Future(f.call(ok).into_future()),
                Err(err) => TryChainAction::Output(Err(err)),
            })
            .map(|x| x.map(|ok| (ok,)))
    }
}
