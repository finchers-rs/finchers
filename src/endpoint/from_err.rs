#![allow(missing_docs)]

use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;


pub fn from_err<E, T, A, B>(endpoint: E) -> FromErr<E::Endpoint, T>
where
    E: IntoEndpoint<A, B>,
    T: From<B>,
{
    FromErr {
        endpoint: endpoint.into_endpoint(),
        _marker: PhantomData,
    }
}


#[derive(Debug)]
pub struct FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

impl<E, T> Endpoint for FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    type Item = E::Item;
    type Error = T;
    type Task = task::FromErr<E::Task, T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        self.endpoint.apply(ctx).map(task::from_err::from_err)
    }
}
