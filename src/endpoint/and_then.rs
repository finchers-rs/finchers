use std::marker::PhantomData;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
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

impl<E, F, R> Endpoint for AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::AndThen<E::Task, fn(E::Item) -> R, F, R>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let f = self.endpoint.apply(ctx)?;
        Some(task::and_then_shared(f, self.f.clone()))
    }
}
