use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
use task::{self, IntoTask};


/// Equivalent to `e.then(f)`
pub fn then<E, F, R, A, B>(endpoint: E, f: F) -> Then<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(Result<A, B>) -> R,
    R: IntoTask,
{
    Then {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `then()`
#[derive(Debug)]
pub struct Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::Then<E::Task, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let fut = self.endpoint.apply(ctx)?;
        Ok(task::then(fut, self.f.clone()))
    }
}
