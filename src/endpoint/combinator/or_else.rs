use futures::{Future, IntoFuture};
use futures::future;

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.or_else(f)`
pub fn or_else<E, F, Fut>(endpoint: E, f: F) -> OrElse<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> Fut,
    Fut: IntoFuture<Item = E::Item>,
{
    OrElse(endpoint, f)
}


/// The return type of `or_else()`
#[derive(Debug)]
pub struct OrElse<E, F>(E, F);

// The implementation of `Endpoint` for `AndThen`.
impl<E, F, Fut> Endpoint for OrElse<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> Fut,
    Fut: IntoFuture<Item = E::Item>,
{
    type Item = Fut::Item;
    type Error = Fut::Error;
    type Future = future::OrElse<E::Future, Fut, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let OrElse(endpoint, f) = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.or_else(f))
    }
}
