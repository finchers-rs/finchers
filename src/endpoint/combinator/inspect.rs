#![allow(missing_docs)]

use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{Poll, Task};


pub fn inspect<E, F>(endpoint: E, f: F) -> Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    Inspect {
        endpoint,
        f: Arc::new(f),
    }
}


#[derive(Debug)]
pub struct Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    endpoint: E,
    f: Arc<F>,
}

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;
    type Task = InspectTask<E, F>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(InspectTask {
            inner,
            f: self.f.clone(),
        })
    }
}


#[derive(Debug)]
pub struct InspectTask<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    inner: E::Task,
    f: Arc<F>,
}

impl<E, F> Task for InspectTask<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.inner.poll(ctx));
        (*self.f)(&item);
        Ok(item.into())
    }
}
