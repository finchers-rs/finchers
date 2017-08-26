use futures::{future, Future};

use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e.map_err(f)`
pub fn map_err<E, F, R>(endpoint: E, f: F) -> MapErr<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> R,
{
    MapErr { endpoint, f }
}


/// The return type of `map_err(e, f)`
pub struct MapErr<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for MapErr<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Future = future::MapErr<E::Future, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let MapErr { endpoint, f } = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.map_err(f))
    }
}
