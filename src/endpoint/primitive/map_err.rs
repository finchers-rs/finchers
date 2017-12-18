use std::marker::PhantomData;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};
use task;



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



#[derive(Debug)]
pub struct MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Task = task::MapErr<E::Task, fn(E::Error) -> R, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::shared::map_err(inner, self.f.clone()))
    }
}
