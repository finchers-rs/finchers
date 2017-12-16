#![allow(missing_docs)]

use std::marker::PhantomData;

use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
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

unsafe impl<E, T> Send for FromErr<E, T>
where
    E: Endpoint + Send,
    T: From<E::Error>,
{
}

unsafe impl<E, T> Sync for FromErr<E, T>
where
    E: Endpoint + Sync,
    T: From<E::Error>,
{
}

impl<E, T> Endpoint for FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    type Item = E::Item;
    type Error = T;
    type Task = task::FromErr<E::Task, T>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::from_err(inner))
    }
}
