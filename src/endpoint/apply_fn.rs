use futures_core::future::TryFuture;

use crate::common::Tuple;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

/// Create an endpoint from a function.
pub fn apply_fn<F, R>(f: F) -> ApplyFn<F>
where
    F: Fn(&mut Context<'_>) -> EndpointResult<R>,
    R: TryFuture<Error = Error>,
    R::Ok: Tuple,
{
    (ApplyFn { f }).with_output::<R::Ok>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ApplyFn<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for ApplyFn<F>
where
    F: Fn(&mut Context<'_>) -> EndpointResult<R> + 'a,
    R: TryFuture<Error = Error> + 'a,
    R::Ok: Tuple,
{
    type Output = R::Ok;
    type Future = R;

    #[inline]
    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (self.f)(ecx)
    }
}
