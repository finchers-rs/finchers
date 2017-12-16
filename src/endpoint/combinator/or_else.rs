use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
use task::{self, IntoTask};


/// Equivalent to `e.or_else(f)`
pub fn or_else<E, F, R, A, B>(endpoint: E, f: F) -> OrElse<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(B) -> R,
    R: IntoTask<Item = A>,
{
    OrElse {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `or_else()`
#[derive(Debug)]
pub struct OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

// The implementation of `Endpoint` for `AndThen`.
impl<E, F, R> Endpoint for OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::OrElse<E::Task, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let task = self.endpoint.apply(ctx)?;
        Ok(task::or_else(task, self.f.clone()))
    }
}
