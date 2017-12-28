#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;

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
    type Task = task::map::Map<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = try_opt!(self.endpoint.apply(ctx));
        Some(task::map::Map {
            task,
            f: self.f.clone(),
        })
    }
}
