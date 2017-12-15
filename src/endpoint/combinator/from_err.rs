#![allow(missing_docs)]

use std::marker::PhantomData;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{Poll, Task};


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
    type Task = FromErrTask<E, T>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(FromErrTask {
            inner,
            _marker: PhantomData,
        })
    }
}


#[derive(Debug)]
pub struct FromErrTask<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    inner: E::Task,
    _marker: PhantomData<T>,
}

impl<E, T> Task for FromErrTask<E, T>
where
    E: Endpoint,
    T: From<E::Error>,
{
    type Item = E::Item;
    type Error = T;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx).map_err(T::from)
    }
}
