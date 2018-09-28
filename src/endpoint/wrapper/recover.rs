use either::Either;
use futures::{Async, Future, Poll};
use http::Response;

use endpoint::{ApplyContext, ApplyResult, Endpoint};
use error::Error;
use output::{Output, OutputContext};

use super::Wrapper;

#[allow(missing_docs)]
pub fn recover<F, R>(f: F) -> Recover<F>
where
    F: Fn(Error) -> R,
    R: ::futures::Future<Error = Error>,
{
    Recover { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Recover<F> {
    f: F,
}

impl<'a, E, F, R> Wrapper<'a, E> for Recover<F>
where
    E: Endpoint<'a>,
    F: Fn(Error) -> R + 'a,
    R: Future<Error = Error> + 'a,
{
    type Output = (Recovered<E::Output, R::Item>,);
    type Endpoint = RecoverEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        RecoverEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RecoverEndpoint<E, F> {
    endpoint: E,
    f: F,
}

impl<'a, E, F, R> Endpoint<'a> for RecoverEndpoint<E, F>
where
    E: Endpoint<'a>,
    F: Fn(Error) -> R + 'a,
    R: Future<Error = Error> + 'a,
{
    type Output = (Recovered<E::Output, R::Item>,);
    type Future = RecoverFuture<E::Future, R, &'a F>;

    fn apply(&'a self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
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
    F1: Future<Error = Error>,
    F2: Future<Error = Error>,
    F: FnOnce(Error) -> F2,
{
    try_chain: TryChain<F1, F2, F>,
}

impl<F1, F2, F> ::futures::Future for RecoverFuture<F1, F2, F>
where
    F1: Future<Error = Error>,
    F2: Future<Error = Error>,
    F: FnOnce(Error) -> F2,
{
    type Item = (Recovered<F1::Item, F2::Item>,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.try_chain
            .poll(|result, f| match result {
                Ok(ok) => TryChainAction::Output(Ok(Either::Left(ok))),
                Err(err) => TryChainAction::Future(f(err)),
            }).map(|x| x.map(|ok| (Recovered(ok),)))
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
    F1: Future<Error = Error>,
    F2: Future<Error = Error>,
{
    Future(F2),
    Output(Result<Either<F1::Item, F2::Item>, Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: Future<Error = Error>,
    F2: Future<Error = Error>,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    #[cfg_attr(feature = "lint", allow(clippy::type_complexity))]
    pub(super) fn poll<F>(&mut self, f: F) -> Poll<Either<F1::Item, F2::Item>, Error>
    where
        F: FnOnce(Result<F1::Item, F1::Error>, T) -> TryChainAction<F1, F2>,
    {
        let mut f = Some(f);

        loop {
            let (out, data) = match self {
                TryChain::First(f1, data) => match f1.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => (Ok(ok), data.take().unwrap()),
                    Err(err) => (Err(err), data.take().unwrap()),
                },
                TryChain::Second(f2) => return f2.poll().map(|x| x.map(Either::Right)),
                TryChain::Empty => panic!("This future has already polled."),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Future(f2) => {
                    *self = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *self = TryChain::Empty;
                    return out.map(Async::Ready);
                }
            }
        }
    }
}
