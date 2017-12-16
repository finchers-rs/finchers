use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
use task;


/// Equivalent to `e.map_err(f)`
pub fn map_err<E, F, R, A, B>(endpoint: E, f: F) -> MapErr<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(B) -> R,
{
    MapErr {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `map_err(e, f)`
#[derive(Debug)]
pub struct MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Task = task::MapErr<E::Task, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::map_err(inner, self.f.clone()))
    }
}
