use futures::{Future, IntoFuture};
use futures::future;

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.then(f)`
pub fn then<E, F, Fut>(endpoint: E, f: F) -> Then<E, F>
where
    E: Endpoint,
    F: FnOnce(Result<E::Item, E::Error>) -> Fut,
    Fut: IntoFuture,
{
    Then { endpoint, f }
}


/// The return type of `then()`
pub struct Then<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, Fut> Endpoint for Then<E, F>
where
    E: Endpoint,
    F: FnOnce(Result<E::Item, E::Error>) -> Fut,
    Fut: IntoFuture,
{
    type Item = Fut::Item;
    type Error = Fut::Error;
    type Future = future::Then<E::Future, Fut, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Then { endpoint, f } = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.then(f))
    }
}
