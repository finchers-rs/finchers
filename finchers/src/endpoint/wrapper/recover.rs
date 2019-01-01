use either::Either;
use http::{Request, Response};

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Async, Context, EndpointFuture, Poll};
use crate::output::IntoResponse;

use super::Wrapper;

#[allow(missing_docs)]
pub fn recover<F, R>(f: F) -> Recover<F>
where
    F: Fn(Error) -> R,
    R: EndpointFuture,
{
    Recover { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Recover<F> {
    f: F,
}

impl<E, F, R> Wrapper<E> for Recover<F>
where
    E: Endpoint,
    F: Fn(Error) -> R + Clone,
    R: EndpointFuture,
{
    type Output = (Recovered<E::Output, R::Output>,);
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

impl<E, F, R> Endpoint for RecoverEndpoint<E, F>
where
    E: Endpoint,
    F: Fn(Error) -> R + Clone,
    R: EndpointFuture,
{
    type Output = (Recovered<E::Output, R::Output>,);
    type Future = RecoverFuture<E::Future, R, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(RecoverFuture {
            try_chain: TryChain::new(f1, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct Recovered<L, R>(Either<L, R>);

impl<L, R> IntoResponse for Recovered<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.into_response(request)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct RecoverFuture<F1, F2, F>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F: FnOnce(Error) -> F2,
{
    try_chain: TryChain<F1, F2, F>,
}

impl<F1, F2, F> EndpointFuture for RecoverFuture<F1, F2, F>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F: FnOnce(Error) -> F2,
{
    type Output = (Recovered<F1::Output, F2::Output>,);

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        self.try_chain
            .try_poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Output(Ok(Either::Left(ok))),
                Err(err) => TryChainAction::Future(f(err)),
            })
            .map(|x| x.map(|ok| (Recovered(ok),)))
    }
}

#[derive(Debug)]
enum TryChain<F1, F2, T> {
    First(F1, Option<T>),
    Second(F2),
    Empty,
}

enum TryChainAction<F1, F2>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
{
    Future(F2),
    Output(Result<Either<F1::Output, F2::Output>, Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    #[cfg_attr(feature = "lint", allow(clippy::type_complexity))]
    pub(super) fn try_poll<F>(
        &mut self,
        cx: &mut Context<'_>,
        f: F,
    ) -> Poll<Either<F1::Output, F2::Output>, Error>
    where
        F: FnOnce(Result<F1::Output, Error>, T) -> TryChainAction<F1, F2>,
    {
        let mut f = Some(f);

        loop {
            let (out, data) = match self {
                TryChain::First(f1, data) => match f1.poll_endpoint(cx) {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => (Ok(ok), data.take().unwrap()),
                    Err(err) => (Err(err), data.take().unwrap()),
                },
                TryChain::Second(f2) => return f2.poll_endpoint(cx).map(|x| x.map(Either::Right)),
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
