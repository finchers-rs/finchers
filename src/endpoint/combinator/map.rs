use futures::{future, Future};

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.map(f)`
pub fn map<E, F, R>(endpoint: E, f: F) -> Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    Map { endpoint, f }
}


/// The return type of `map(e, f)`
pub struct Map<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    type Item = R;
    type Future = future::Map<E::Future, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Map { endpoint, f } = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.map(f))
    }
}
