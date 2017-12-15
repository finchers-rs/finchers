#![allow(missing_docs)]

use std::marker::PhantomData;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task;


pub fn from_err<E, T>(endpoint: E) -> FromErr<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    FromErr {
        endpoint,
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

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::from_err(inner))
    }
}
