#![allow(missing_docs)]

use std::marker::PhantomData;
use futures::{future, Future};

use context::Context;
use endpoint::{Endpoint, EndpointResult};


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
pub struct FromErr<E, T> {
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
    type Future = future::FromErr<E::Future, T>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let f = self.endpoint.apply(ctx)?;
        Ok(f.from_err())
    }
}
