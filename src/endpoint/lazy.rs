use futures::{Future, IntoFuture, Poll};

use endpoint::{ApplyContext, ApplyResult, Endpoint};
use error::Error;

/// Create an endpoint from the specified function which returns a `Future`.
pub fn lazy<F, R>(f: F) -> Lazy<F>
where
    F: Fn() -> R,
    R: IntoFuture<Error = Error>,
{
    Lazy { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Lazy<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for Lazy<F>
where
    F: Fn() -> R + 'a,
    R: IntoFuture<Error = Error> + 'a,
{
    type Output = (R::Item,);
    type Future = LazyFuture<R::Future>;

    fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(LazyFuture {
            future: (self.f)().into_future(),
        })
    }
}

#[derive(Debug)]
pub struct LazyFuture<F> {
    future: F,
}

impl<F> Future for LazyFuture<F>
where
    F: Future<Error = Error>,
{
    type Item = (F::Item,);
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map(|x| x.map(|ok| (ok,)))
    }
}
