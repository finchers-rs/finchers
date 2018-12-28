use futures::{Future, IntoFuture, Poll};

use common::Tuple;
use endpoint::{ApplyContext, ApplyResult, Endpoint};
use error::Error;

/// Create an endpoint from a function which takes the reference to `ApplyContext`
/// and returns a future.
///
/// The endpoint created by this function will wrap the result of future into a tuple.
/// If you want to return the result without wrapping, use `apply_raw` instead.
pub fn apply<F, R>(f: F) -> Apply<F>
where
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<R>,
    R: IntoFuture<Error = Error>,
{
    (Apply { f }).with_output::<(R::Item,)>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Apply<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for Apply<F>
where
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<R> + 'a,
    R: IntoFuture<Error = Error> + 'a,
{
    type Output = (R::Item,);
    type Future = ApplyFuture<R::Future>;

    #[inline]
    fn apply(&'a self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (self.f)(ecx).map(|res| ApplyFuture(res.into_future()))
    }
}

#[derive(Debug)]
pub struct ApplyFuture<F>(F);

impl<F> Future for ApplyFuture<F>
where
    F: Future<Error = Error>,
{
    type Item = (F::Item,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map(|x| x.map(|ok| (ok,)))
    }
}

/// Create an endpoint from a function which takes the reference to `ApplyContext`
/// and returns a future *without wrapping its result into a tuple*.
pub fn apply_raw<F, R>(f: F) -> ApplyRaw<F>
where
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<R>,
    R: IntoFuture<Error = Error>,
    R::Item: Tuple,
{
    (ApplyRaw { f }).with_output::<R::Item>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ApplyRaw<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for ApplyRaw<F>
where
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<R> + 'a,
    R: IntoFuture<Error = Error> + 'a,
    R::Item: Tuple,
{
    type Output = R::Item;
    type Future = R::Future;

    #[inline]
    fn apply(&'a self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        (self.f)(ecx).map(IntoFuture::into_future)
    }
}
