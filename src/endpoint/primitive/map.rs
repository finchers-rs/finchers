use std::marker::PhantomData;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};
use task;


/// Equivalent to `e.map(f)`
pub fn map<E, F, R, A, B>(endpoint: E, f: F) -> Map<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(A) -> R,
{
    Map {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `map(e, f)`
#[derive(Debug)]
pub struct Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;
    type Task = task::Map<E::Task, fn(E::Item) -> R, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::shared::map(inner, self.f.clone()))
    }
}
