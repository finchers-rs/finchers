#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
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
    type Task = task::map_err::MapErr<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = try_opt!(self.endpoint.apply(ctx));
        Some(task::map_err::MapErr {
            task,
            f: self.f.clone(),
        })
    }
}
