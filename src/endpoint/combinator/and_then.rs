use futures::{Future, IntoFuture};
use futures::future;

use context::Context;
use errors::FinchersError;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.and_then(f)`
pub fn and_then<E, F, Fut, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Item = R, Error = FinchersError>,
{
    AndThen(endpoint, f)
}


/// The return type of `and_then()`
pub struct AndThen<E, F>(E, F);

// The implementation of `Endpoint` for `AndThen`.
impl<E, F, Fut, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Item = R, Error = FinchersError>,
{
    type Item = R;
    type Future = future::AndThen<E::Future, Fut, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let AndThen(endpoint, f) = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.and_then(f))
    }
}
