#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;
use futures::IntoFuture;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;

pub fn then<E, F, R, A, B>(endpoint: E, f: F) -> Then<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(Result<A, B>) -> R,
    R: IntoFuture,
{
    Then {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::then::Then<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = try_opt!(self.endpoint.apply(ctx));
        Some(task::then::Then {
            task,
            f: self.f.clone(),
        })
    }
}
