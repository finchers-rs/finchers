#![allow(missing_docs)]

use std::marker::PhantomData;
use std::sync::Arc;
use futures::IntoFuture;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;

pub fn or_else<E, F, R, A, B>(endpoint: E, f: F) -> OrElse<E::Endpoint, F, R>
where
    E: IntoEndpoint<A, B>,
    F: Fn(B) -> R,
    R: IntoFuture<Item = A>,
{
    OrElse {
        endpoint: endpoint.into_endpoint(),
        f: Arc::new(f),
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<fn() -> R>,
}

impl<E, F, R> Endpoint for OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoFuture<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = task::or_else::OrElse<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = try_opt!(self.endpoint.apply(ctx));
        Some(task::or_else::OrElse {
            task,
            f: self.f.clone(),
        })
    }
}
