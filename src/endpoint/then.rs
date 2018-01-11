#![allow(missing_docs)]

use std::fmt;
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

impl<E, F, R> Clone for Then<E, F, R>
where
    E: Endpoint + Clone,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoFuture,
{
    fn clone(&self) -> Self {
        Then {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E, F, R> fmt::Debug for Then<E, F, R>
where
    E: Endpoint + fmt::Debug,
    F: Fn(Result<E::Item, E::Error>) -> R + fmt::Debug,
    R: IntoFuture,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Then")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
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
