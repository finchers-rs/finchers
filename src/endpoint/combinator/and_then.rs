use futures::{Future, IntoFuture};
use futures::future;

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.and_then(f)`
pub fn and_then<E, F, Fut>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Error = E::Error>,
{
    AndThen { endpoint, f }
}


/// The return type of `and_then()`
#[derive(Debug)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, Fut> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Error = E::Error>,
{
    type Item = Fut::Item;
    type Error = Fut::Error;
    type Future = future::AndThen<E::Future, Fut, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let AndThen { endpoint, f } = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.and_then(f))
    }
}
