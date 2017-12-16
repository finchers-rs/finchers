use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
use task::{self, IntoTask};


/// Equivalent to `e.and_then(f)`
pub fn and_then<E, F, R, A, B>(endpoint: E, f: F) -> AndThen<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(A) -> R,
    R: IntoTask<Error = B>,
{
    AndThen {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `and_then()`
#[derive(Debug)]
pub struct AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

unsafe impl<E, F, R> Send for AndThen<E, F, R>
where
    E: Endpoint + Send,
    F: Fn(E::Item) -> R + Send,
    R: IntoTask<Error = E::Error>,
{
}

unsafe impl<E, F, R> Sync for AndThen<E, F, R>
where
    E: Endpoint + Sync,
    F: Fn(E::Item) -> R + Sync,
    R: IntoTask<Error = E::Error>,
{
}

impl<E, F, R> Endpoint for AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::AndThen<E::Task, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let f = self.endpoint.apply(ctx)?;
        Ok(task::and_then(f, self.f.clone()))
    }
}
