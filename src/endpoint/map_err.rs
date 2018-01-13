#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use super::{Endpoint, EndpointContext, IntoEndpoint};
use super::task;

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

pub struct MapErr<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Clone for MapErr<E, F, R>
where
    E: Endpoint + Clone,
    F: Fn(E::Error) -> R,
{
    fn clone(&self) -> Self {
        MapErr {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E, F, R> fmt::Debug for MapErr<E, F, R>
where
    E: Endpoint + fmt::Debug,
    F: Fn(E::Error) -> R + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
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
