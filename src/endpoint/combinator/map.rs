use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
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
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for Map<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;
    type Task = task::Map<E::Task, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::map(inner, self.f.clone()))
    }
}
