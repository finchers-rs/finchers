use futures::future::IntoFuture;

use crate::common::Tuple;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

/// Create an endpoint from a function.
pub fn apply_fn<F, R>(f: F) -> ApplyFn<F>
where
    F: Fn(&mut Context<'_>) -> EndpointResult<R>,
    R: IntoFuture<Error = Error>,
    R::Item: Tuple,
{
    (ApplyFn { f }).with_output::<R::Item>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ApplyFn<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for ApplyFn<F>
where
    F: Fn(&mut Context<'_>) -> EndpointResult<R> + 'a,
    R: IntoFuture<Error = Error> + 'a,
    R::Item: Tuple,
{
    type Output = R::Item;
    type Future = R::Future;

    #[inline]
    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (self.f)(ecx).map(IntoFuture::into_future)
    }
}
