#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;
use futures::IntoFuture;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;


pub fn and_then<E, F, R, A, B>(endpoint: E, f: F) -> AndThen<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(A) -> R,
    R: IntoFuture<Error = B>,
{
    AndThen {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoFuture<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::and_then::AndThen<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = self.endpoint.apply(ctx)?;
        Some(task::and_then::AndThen {
            task,
            f: self.f.clone(),
        })
    }
}
